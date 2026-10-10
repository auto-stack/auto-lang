---
plan_id: PLAN-752
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: pg-bootstrap-lib
author: [agent]
created_at: 2026-10-09
updated_at: 2026-10-09
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [auto-playground, auto-lang, playground-vue]
current_step: 5
total_steps: 5
---

# [PLAN-752] playground bootstrap 语料可运行化（lib-legacy 前置拼接）

## 0. 变更摘要

让 playground 的 `vm-bootstrap` 组语料（99_bootstrap，AAVM v1 自举材料，~105 条
笔记）在独立 playground（后端自服务 SPA + website NotesExplorer）中可运行：
`/api/run` 新增 `prepend_lib` 参数——为真时服务端按 golden 管线 `is_bootstrap`
同款清单（`AUTO_LIB_FILES`，12 个 `auto/lib-legacy/*.at`，~188KB）拼接源码后编译
运行；前端宿主按笔记分组（note.id 前缀 `vm-bootstrap/`）自动开启。

2026-10-09 端到端预验证（运行中 playground 服务实测）：`a2r_hello` 裸跑报
`Undefined symbol: run_a2r`；前置 lib-legacy 12 文件后成功输出 `ok`
（与其 `.expected.out` 一致）。

**边界调查结论（scope 依据）**：
- aavm-m1/m2 的失败语料（如 `c01_arith`）与 lib 无关——v2 前置/裸跑同败于
  `let` 不可重赋值语义，属 aavm 语料自有隔离问题，**不在本计划范围**；
- aavm-use 失败为多文件模块用例（temp-dir 物化解析），**不在本计划范围**；
- 故本计划只覆盖 `vm-bootstrap` 组（走查实测 98 条报错）。

## 1. 目标

- G-1：playground 运行 vm-bootstrap 笔记时，符号解析完整（不再
  `Undefined symbol`），输出与 golden `.expected.out` 一致。
- G-2：行为默认关闭、按笔记来源显式开启——非 bootstrap 笔记零影响
  （性能与语义双重：拼接使编译输入 ~+188KB）。
- G-3：不破坏 PLAN-746 的内存/超时保护（deadline 照常生效）。

**非目标**：
- 不改 golden 测试管线（`run_vm_file_test`/`is_bootstrap` 维持现状）。
- 不处理 aavm 组语料（语义/模块问题归 aavm 线）。
- 不做 v2 lib（`AUTO_LIB_FILES_V2`）前置通道——v2 接口与 v1 语料不兼容
  （预验证实测），且 AAVM v2 归 aavm 线。
- 不优化拼接编译耗时（实测 1-2s 内可接受）。

**受影响**：本仓 `crates/auto-lang`（lib.rs 公开面）、`crates/auto-playground`
（run.rs/vm_runner.rs）、`packages/auto-playground-vue`（类型+两 composable）、
`crates/auto-playground/frontend`（App.vue 宿主判定）、`docs/specs/auto-playground/`。

## 2. 架构方案

```
笔记选中(App.vue/NotesExplorer)
  └─ note.id 前缀 "vm-bootstrap/" → noteMeta.prependLib = true
AutoPlaygroundFull(noteMeta) → usePlaygroundFull.prependLib
  └─ projectRequestBody: body.prepend_lib = prependLib
POST /api/run {source, prepend_lib, timeout_secs}
  └─ vm_runner::run_source_with_lib(source, deadline)
       └─ auto_lang::bootstrap_lib_source()  (OnceLock 缓存, AUTO_LIB_FILES 序)
       source' = lib + "\n" + source
  execute_autovm_with_deadline(source', ...)   ← PLAN-746 管线不变
```

- lib 来源单一事实源 = 既有 `pub(crate) const AUTO_LIB_FILES`（lib.rs:2001，
  12 文件，依赖序 pos→token→error→lexer→ast→parser→typeinfer→codegen→vm→
  a2r→generics→eval）。新增 `pub fn bootstrap_lib_source() -> AutoResult<String>`
  （OnceLock 缓存拼接结果；逐文件容错缺失=跳过，与 `read_auto_lib` 同款语义）。
- 拼接只在 `prepend_lib=true` 且 `project_dir` 为空时作用于裸 source 路径
  （project/files 路径不受影响——bootstrap 笔记全是 single kind）。
- 前端判定放在宿主（App.vue / NotesExplorer 侧），组件只透传 noteMeta 新字段
  `prependLib?: boolean`——website 宿主后续接入时同口径。

## 3. 技术栈

Rust（auto-lang/lib.rs、auto-playground）、Vue3/TS（组件包+宿主）、
验证＝既有走查脚本（`scratch/playground-check/run_all_examples_mem.py`）+
vm-golden `.expected.out` 比对 + e2e。

## 4. 需求分析与背景调查

**授权**：用户 2026-10-09 "OK，按流程立项做掉"——标准四技能流程 +
worktree（lang-752 组），范围以本合同为准。

**预验证证据**（2026-10-09，主检出终版二进制 playground :3030）：
- 裸跑 `a2r_hello` → `Error: Undefined symbol: run_a2r`；
- 前置 v2 `auto/lib`（473KB，7 文件）→ 仍 `Undefined symbol`（接口代差）；
- 前置 lib-legacy 11 文件（漏 eval.at）→ `Undefined symbol: eval_str_cat`；
- 前置 lib-legacy **12 文件（188KB，AUTO_LIB_FILES 全清单）→ 输出 `ok`，
  与 `test/vm/99_bootstrap/.../a2r_hello.expected.out` 一致**。
- `c01_arith`（aavm-m1）v2 前置与裸跑同败于 `let` 语义 → 非 lib 问题。

**代码锚点**：
- `crates/auto-lang/src/lib.rs:2001` AUTO_LIB_FILES；`:2052` read_auto_lib
  （私有，pub(crate) 常量）；`:2017` run_vm_file_test 的 is_bootstrap 拼接
  先例（`format!("{}\n{}", lib, src)`）。
- `crates/auto-playground/src/routes/run.rs` RunRequest（PLAN-746 已加
  timeout_secs）；`vm_runner.rs` run_source(source, deadline)。
- `packages/auto-playground-vue/src/composables/usePlaygroundFull.ts:83`
  projectRequestBody（PLAN-746 已发 timeout_secs）；`AutoPlaygroundFull.vue:79`
  noteMeta prop；`crates/auto-playground/frontend/src/App.vue:44` noteMeta 构造
  （有完整 note.id 可用）。
- golden 期望输出：`crates/auto-lang/test/vm/99_bootstrap/<case>/<name>.expected.out`。

## 5. 详细设计

### lib 公开面（auto-lang）

```rust
// lib.rs（AUTO_LIB_FILES 旁）
static BOOTSTRAP_LIB: OnceLock<Option<String>> = OnceLock::new();
/// PLAN-752: AAVM v1 自举 lib（AUTO_LIB_FILES 全清单拼接，依赖序），
/// 供 playground prepend_lib 使用；首次调用后缓存。文件缺失跳过（同 read_auto_lib）。
pub fn bootstrap_lib_source() -> Option<&'static str>
```
缺失全部文件时返回 None（调用方按未开启处理并报错日志）。

### playground 接线

- `RunRequest` 增 `prepend_lib: Option<bool>`（默认 None/false）。
- `vm_runner::run_source(source, deadline)` 增内部前置：`let source = if
  prepend_lib { format!("{}\n{}", lib, source) }`；仅裸 source 路径生效。
- `run_handler` 解析并贯通（与 timeout_secs 同节奏）。

### 前端

- `NoteMeta` 类型增 `prependLib?: boolean`；`AutoPlaygroundFull` noteMeta
  prop 类型同步；`usePlaygroundFull` 持 `prependLib` ref（默认 false，
  noteMeta 变化时同步），projectRequestBody 置 `body.prepend_lib`。
- `frontend/src/App.vue`：noteMeta 构造加
  `prependLib: n.id.startsWith("vm-bootstrap/")`。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-playground/project.md | before：/api/run 只跑裸 source；after：`prepend_lib: bool`（默认 false）——按 AUTO_LIB_FILES（lib-legacy 12 文件）前置拼接，仅裸 source 路径生效；宿主对 vm-bootstrap 组笔记自动开启 | 语料可运行化；golden is_bootstrap 同款清单保语义一致 | AC-01, AC-02 |
| SD-02 | add | docs/specs/playground-vue/project.md | before：noteMeta 无执行修饰；after：noteMeta.prependLib 透传契约（宿主按 note.id 组前缀判定） | 前后端契约 | AC-03 |

## 6. 测试设计

- **单测（auto-lang）**：`bootstrap_lib_source()` 非空、含 eval.at 内容
  （`eval_str_cat` 符号在库中）、二次调用同引用（缓存）。
- **服务端 e2e（playwright 或 curl 级）**：`a2r_hello` 笔记体 +
  `prepend_lib:true` → stdout 与 `.expected.out` 一致；同请求不带
  prepend_lib → Undefined symbol（负面对照）。
- **全量走查**：修复后跑 `run_all_examples_mem.py`（已含 60s 与 stdout
  Error 判定）——vm-bootstrap 组报错应从 ~98 降至个位数（残留逐条归因：
  预期 error 语料/超时兜底属正确行为）。
- **PLAN-746 回归**：plan746 单测 8 条 + run-timeout e2e 5 条 +
  demo 组 28 条全绿（内存保护不破）。

## 7. 验收标准

| id | 标准 | 验证方法 |
|---|---|---|
| AC-01 | `a2r_hello`（及抽样 ≥10 条 vm-bootstrap 笔记）带 prepend_lib 运行时输出与 golden `.expected.out` 逐字节一致 | 定向 curl 比对 + 全量走查 expected_match 统计 |
| AC-02 | 非 bootstrap 笔记零影响：demo 组 28 条全绿；裸 source 不带 prepend_lib 的 a2r_hello 仍报 Undefined symbol（行为不变）；拼接仅 prepend_lib=true 时发生 | 走查对照 + 负面对照 curl |
| AC-03 | 前端宿主自动开启：选中 vm-bootstrap 笔记时 /api/run 请求体含 prepend_lib=true；选其他组笔记不含或为 false | e2e 拦截断言（仿 PLAN-746 timeout_secs 拦截测试） |
| AC-04 | PLAN-746 保护不回归：plan746 单测 8/8、run-timeout e2e 5/5、走查后服务端净增 <300MB | 既有测试档 + 走查内存曲线 |
| AC-05 | 复审门禁：裸 `cargo t` 红集无新增、`cargo tv` 全绿；spec 增量落库 | 复审档 |

## 8. 执行步骤

- [x] T-01 auto-lang 公开面：`bootstrap_lib_source()`（OnceLock 缓存，
  AUTO_LIB_FILES 序拼接，缺失跳过）。文件：crates/auto-lang/src/lib.rs。
  验证：plan752_bootstrap_lib_source_present_and_cached PASS（>100KB/含 eval_str_cat/
  缓存同一性）；commit 已在 fc76d2fb4 链。AC-01
- [x] T-02 playground 接线：RunRequest.prepend_lib → run_handler →
  vm_runner（仅裸 source 路径拼接）。文件：crates/auto-playground/src/
  routes/run.rs、vm_runner.rs。验证：`cargo check -p auto-playground` +
  定向 curl 正负对照（a2r_hello 带/不带 prepend_lib）。AC-01, AC-02
- [x] T-03 前端契约：NoteMeta.prependLib + usePlaygroundFull 透传 +
  App.vue 组前缀判定。文件：packages/auto-playground-vue（types/
  AutoPlaygroundFull/usePlaygroundFull）、frontend/src/App.vue。
  验证：dist 重建 + e2e 拦截断言 2 条（bootstrap 发 true/其他组不发）——
  注意 worktree 需补 frontend/public/playground-data/notes.json（gitignored），
  缺失时侧栏空树（e2e 首败环境因）。AC-03
- [x] T-04 全量走查复验：run_all_examples_mem.py 全量——vm-bootstrap 组
  报错降至个位数并逐条归因；内存曲线平坦；demo 组全绿。
  验收档：scratch/playground-check/p752-report.md（worktree）。vm-bootstrap 105 条：
  104 成功/80 golden 逐字节一致/24+1 余红全部=仓库既有 #[ignore] 隔离态
  （vm_file_tests.rs:1773+ 与 aavm_runner_tests.rs 逐条 #[ignore] 实证）；
  demo 28/28；内存 32→63MB（峰值 127MB 回落，无泄露）。AC-01, AC-02, AC-04
- [x] T-05 复审门禁：plan746 单测、run-timeout e2e、裸 `cargo t` 红集对拍、
  `cargo tv`；spec 增量落库（merge 档）。AC-04, AC-05
  实录：plan746 9/9、plan752 1/1、tv 162/162、e2e 7/7、红集对拍 14=14
  （互差 plan484/plan707×2 为并行负载 flake，隔离复跑绿——同 PLAN-746 复审结论）

## 9. 复审记录

- 2026-10-09 `PLAN-752:r1 merge 收据 | prepared(e194481f6→rebase 5b3b4e257,range-diff=) |
  landed(master tip=5b3b4e257,ff-only;主检出产物重建:playground 二进制+frontend dist) |
  ledger_refreshed(.autoos/specs.json designs 段 P752-1/P752-2,docsha 冻结;INDEX 26 projects;
  auto-playground/playground-vue plans.md 回写;README §5 手工回退) | archived | cleaned(见最终回执) |
  batch_regression: 752%5≠2;due 判定见 .last-batch-regression.json | completion_kind: delivered`
- 2026-10-09 `stage: review | PLAN-752 | r1 | outcome: pass | reviewed e194481f6
  （code tip fc76d2fb4）| base c5adecd1b | dep auto-down @895f8d0 | spec_inputs:
  docs/specs/auto-playground/project.md（SD-01）+ playground-vue/project.md（SD-02），
  docsha 随 merge 冻结 | acceptance_results: AC-01 pass（全量 80/105 逐字节一致；
  抽样复现 9/12，3 条不一致逐条实证为 #[ignore] 隔离语料）/AC-02 pass（负面对照
  Undefined symbol + demo 28/28）/AC-03 pass（e2e 拦截 2/2 live 重跑）/AC-04 pass
  （plan746 9/9 + 走查 +31MB 峰值回落无泄露）/AC-05 pass（红集 14=14，互差 2 条
  负载 flake 隔离绿；tv 162/162）| findings: 无（SD-01 笔误修正 e194481f6，docs-only）|
  evidence: p752-report.md + 本轮 curl/截图实录 | next: merge`
- 2026-10-09 `stage: work | PLAN-752 | r1 | outcome: pass | code plan-752-dev @ fc76d2fb4
  （+spec 预埋）| tasks T-01..T-05 全完成 | evidence：单测/tv/e2e/全量走查（p752-report.md）/
  红集对拍零新增 | blockers: 无 | next: review`
- 2026-10-09 `/auto-plan:new` r1 起草：`stage: new`，PLAN-752 revision 1。
  `outcome: pass`（用户已批"按流程立项做掉"，授权内可开工）。
  `next: work`。已定谳：范围=vm-bootstrap 组 only（aavm 组非 lib 问题，预验证
  实证）；lib 清单=lib-legacy AUTO_LIB_FILES（v2 不兼容实证）。

## 10. 待澄清事项

- Q-1：website 宿主（NotesExplorer→PlaygroundCard）是否同步接 prependLib？
  本计划先交付独立 SPA 宿主（App.vue）；website 宿主接入为后续一行透传，
  不在验收内（若 PlaygroundCard 链路低成本可顺带，T-03 内记录实际范围）。

## spec-sync 回写记录

- `docs/specs/auto-playground/project.md`：PLAN-752 节（prepend_lib 语义/清单/宿主自动开启/走查结果）。
- `docs/specs/playground-vue/project.md`：PLAN-752 节（noteMeta.prependLib 透传契约）。
- `docs/specs/auto-playground/plans.md` + `docs/specs/playground-vue/plans.md`：752 行一句话沉淀。
- `.autoos/specs.json` designs 段：P752-1、P752-2（commit:5b3b4e257 + docsha 冻结）。
- `docs/specs/INDEX.md`：spec-index.py 再生。
