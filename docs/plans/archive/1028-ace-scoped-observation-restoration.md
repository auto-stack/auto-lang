---
plan_id: PLAN-1028
status: archived
completion_kind: delivered
feature_name: ACE计量窗口嵌套与unwind恢复修复
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-13
plan_revision: 1
current_step: 6
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: [docs/specs/auto-ac/closures-function-production.md]
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1028] ACE计量窗口嵌套与unwind恢复修复

## 0. 变更摘要

承接[1024](../1024-ace-m2-code-quality-and-refactoring-audit.md)的F02，优先级**P1**。2026-11-13 `/goal 完成计划1028` 端到端交付：于专属 worktree 提交 `be7f1554d` 统一 `ace_core::capture_scoped` RAII `Drop` 恢复与内层隔离语义并完成 `OBS-01`/`OBS-02` + 五固定组并集 2336/2336 PASS 门禁与独立复审；rebase 到最新 `master`（`9a7674009`，仅含 `1017` 文档提交，ACE 代码零变更）并经 `git range-diff` 1:1 等价验证与 `ace-core`/`ace-native` 重跑通过后，fast-forward 合入 `master` 为 **`7880f2a425b535a9aee2ab97c43106c604de5842`**，规范 `closures-function-production.md` 落地核验一致，现归档收尾。

## 1. 目标

观察窗口在嵌套/正常返回/Err/panic unwind后恢复外层状态；无窗口不产生幽灵记录。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。不改变正式计量样本/预算，不由观察修复反推历史验收无效；内层是否累加如与已承诺接口冲突，先T-01记录决定。

## 2. 架构方案

沿现有Drop/Scope机制，先将同类窗口内层隔离语义与stage/type现行为对齐；panic仍传播，不吞异常变PASS；尝试计数和接受态分开。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。等价机制由work选择；公共语义/支持/跨核心合同变化先具名设计及修约，不能临时加深度/规模限制。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。唯一工作树 `D:/autostack/.wt/lang-1028/auto-lang`（分支 `plan-1028-dev`，基面 master `c2760324d`；兄弟 `auto-down` detached `58ea33f`，零链接）。材料唯一 `D:/autostack/plan-artifacts/plan-1028/`（已登记于 `.retention/policy.json` 为 `in-flight`）。

## 4. 需求分析与背景调查

授权：2026-11-13 `/goal 完成计划1028`（`/auto-plan-goal`）一次性授权本计划在专属 worktree 完整执行 work、review、有界修复与 merge 收尾（不改变验收/范围/兼容/仓库边界）。
T-01 基线与漂移核查（master `c2760324d`）：
- `experimental/ace/crates/core/src/lib.rs` SHA256 `906442f4d5f2ab0f585c37e7ce44965914bf521e706e7a62fc0ca648482ea48d`（与 1024 基线一致）。
- `experimental/ace/crates/core/tests/cases/closures_fn_912.rs` SHA256 `954b829d2c5202bc83f5a2836511f2b57674ebc1dc2e440fc6ea7eaa559d6503`（一致）。
- `docs/specs/auto-ac/closures-function-production.md` SHA256 `8e29772c3dea9f138e055912b134ca957267e35496dde958b7b4ef8bdbb4321f`（一致）。
- `experimental/ace/crates/core/src/source/parser.rs` SHA256 `0478200351b0a174df16362222b4f50fc20f27b49406ba38b287f164a85042d8`（较 `81d8fc03` 漂移源于 PLAN-963/console 语法条目，`capture_syntax_work` L77–92 未变且已含 `Drop` 恢复）。

T-01 策略与消费者冻结决定：
1. **内层隔离对齐**：`capture_stage_attempts`、`type_synth::capture` 与 `capture_syntax_work` 已采用内层隔离（内层仅返回自身事件，外层保留 `a+c`）；全仓消费者（`adapter.rs`、`closures_fn_session.rs`、`data_session.rs`、`closures_incremental.rs`、`collections_incremental.rs`、`incremental.rs`、`source-query`、`a2x-worker`）均以单窗口度量本次生产，无累加契约。故 `capture_body_work` 统一对齐内层隔离 + RAII `Drop` 恢复，不改变公共签名与既有消费语义。
2. **受影响固定组并集**：除原拟 `ace-source`、`ace-native`、`ace-av` 外，经符号全仓检索，`ace-source-query`（`ace-semantics`）与 `ace-a2x-worker`（`ace-a2x`）亦直接调用 `capture_body_work`/`capture_stage_attempts`，故冻结五组并集：`ace-source ace-native ace-av ace-semantics ace-a2x`。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/core/src/lib.rs](../../../experimental/ace/crates/core/src/lib.rs) | 906442f4d5f2ab0f585c37e7ce44965914bf521e706e7a62fc0ca648482ea48d | 静态取材，非运行证据 |
| [experimental/ace/crates/core/src/source/parser.rs](../../../experimental/ace/crates/core/src/source/parser.rs) | 7956a5e367b344f5ca14005fc3499a8e3eea9a4e1928dabbe094ee68cd002f8c | 静态取材，非运行证据 |
| [experimental/ace/crates/core/tests/cases/closures_fn_912.rs](../../../experimental/ace/crates/core/tests/cases/closures_fn_912.rs) | 954b829d2c5202bc83f5a2836511f2b57674ebc1dc2e440fc6ea7eaa559d6503 | 静态取材，非运行证据 |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-ac/closures-function-production.md](../../../docs/specs/auto-ac/closures-function-production.md) | 8e29772c3dea9f138e055912b134ca957267e35496dde958b7b4ef8bdbb4321f | 已按 SD-01 更新并核验（新 SHA256 `6ef4305d...`） |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。

## 5. 详细设计

唯一范围=F02。关闭行必须连接“发现→现代码不变量→反例→真实consumer→永久断言→修复提交”，承接登记不算关闭。T-01使§6全部case在原入口可运行。

数据/错误/责任：观察窗口在嵌套/正常返回/Err/panic unwind后恢复外层状态；无窗口不产生幽灵记录。 沿现有Drop/Scope机制，先将同类窗口内层隔离语义与stage/type现行为对齐；panic仍传播，不吞异常变PASS；尝试计数和接受态分开。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 闭合记录（F02）

| 发现 | 不变量 | 受影响路径/状态 | 修复与永久断言 | 当前结果 |
|---|---|---|---|---|
| F02 | 同类嵌套保真（内层隔离、外层 `a+c`）；`Result::Err`/panic unwind/无外层均由 RAII `Drop` 恢复，panic 照常传播，无幽灵记录 | `ace_core::body_work::{capture_body_work, capture_stage_attempts}`、`ace_core::type_synth::capture`（`experimental/ace/crates/core/src/lib.rs` `capture_scoped`）；真实消费者 `ClosuresNativeSession::new` 等 | 抽取 `pub(crate) fn capture_scoped`（`Restore` + `Drop` `try_with`）；永久回归 `ace-core` `observation_scope_1028::obs_01_*`/`obs_02_*` 与 `ace-native` `observation_scope_1028::obs_02_native_*`（commit `be7f1554d`） | 修前红已复现；修后 3/3 新增用例及 5 固定组 2336/2336 用例 PASS |

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify（已备于 worktree） | docs/specs/auto-ac/closures-function-production.md | 现行合同保持；增补「观察窗口恢复边界（PLAN-1028 / F02）」，记录三窗口 RAII `Drop` 恢复、内层隔离、panic 传播、零幽灵及 `ace-core`/`ace-native` 回归入口 | 观察窗口在嵌套/正常返回/Err/panic unwind后恢复外层状态；无窗口不产生幽灵记录。 | AC-02/03/04 |

`docs/specs/auto-ac/closures-function-production.md` 已在 worktree 准备并提交于 `be7f1554d`（SHA256 `6ef4305d2f2f80ec5dec50eb3408751f36bab9e843f845d8dff392c3e9be4bab`）；待 review/merge 核验合入；`touched_goals` 空，工程质量不擅扩产品目标。

## 6. 测试设计

冻结受影响五组并集：`cargo t ace-source ace-native ace-av ace-semantics ace-a2x`（`config_sha256=e8b55f0eeecc01bb4eb2c5d672a11c2113ee00a39aae8842d81b3f650f6c7f2f`，`source_state=77046e8a31bf8270aa9f7d30b5b908cf0c11404570ff198b871ef8afca370f7e`）。原样本/规模/平台/ignore/正式性能轴保持；merge 可复用同身份有效证据，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| OBS-01 | AC-02 / T-02 | body包body，外层a/内层b/外层c；stage/type同类窗口；异类五窗口及无窗口前后 | `ace-core` `observation_scope_1028::obs_01_nested_same_kind_body_stage_type_and_cross_windows` | 内层仅b，外层a+c；不同窗口独立，无遗失/重复/幽灵 | 修前红（L59 `outer_a` 丢失 `None`）；修后绿（`be7f1554d`，0.25s PASS） |
| OBS-02 | AC-02 / T-02 | `catch_unwind` 捕获内层/顶层panic后继续观察；`Result::Err` 正常返回；真实 native 冷启动 | `ace-core` `observation_scope_1028::obs_02_*` + `ace-native` `observation_scope_1028::obs_02_native_*` | 外层恢复；下一窗口干净；原panic可见；Err不被误当unwind；`ClosuresNativeSession::new` 嵌套/panic后恢复 exit=42 | 修前红（core L221 `outer_a` `None`；native L60 `outer_a` `{}`）；修后绿（`be7f1554d`，core 0.07s / native 7.56s PASS） |

T-03 冻结五组并集实测结果（`cargo t ace-source ace-native ace-av ace-semantics ace-a2x`，total wall 1625.32s，`gate_passed: True`，`results.json` 已归集于 `plan-artifacts/plan-1028/cargo-t-union-results.json`）：
- `ace-source`: PASS（502 executed / 502 selected；excluded=1 `run_exe_60s_deadline_bounds_descendant_pipe_hold` ignored；not_run=0；prep 27.2s / exec 306.2s；child_valid=True）
- `ace-native`: PASS（248 executed / 248 selected；excluded=0；not_run=0；prep 27.2s / exec 93.8s；child_valid=True）
- `ace-av`: PASS（1088 executed / 1088 selected；excluded=0；not_run=0；prep 41.0s / exec 52.3s；child_valid=True）
- `ace-semantics`: PASS（269 executed / 269 selected；excluded=0；not_run=0；prep 13.4s / exec 4.3s；child_valid=True）
- `ace-a2x`: PASS（229 executed / 229 selected；excluded=2 `inventory_941::read_{composite,scalar}_contract_inventory_941`；not_run=0；prep 16.0s / exec 354.5s；child_valid=True）

角色：work（本会话执行）；review（本会话独立复审路径：按 `auto-plan-review` 逐源码/Diff/证据重核并记录单会话局限，Plan 未指派独立最终复审人）；merge（本会话收尾）。使用同一实施树 `D:/autostack/.wt/lang-1028/auto-lang`。

## 7. 验收标准

- [x] AC-01 / work：T-01冻结真实源、支持/资源边界与前置，所有§6反例及邻例可在实际入口运行，重大未决先定，不私改expected。
- [x] AC-02 / work：§6每行独立oracle已取得；缺口修复或有充分反证，修前/修后及同族边界证据齐，原义务不减。
- [x] AC-03 / work：消费者、原测试、恢复/重试/失败责任保持，完成适用门、Spec准备及1002依赖分流，未闭不冒PASS。
- [x] AC-04 / review：逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。
- [x] AC-05 / merge：review后冻结并集门/正式合入、规范沉淀及归档真实完成。
- [x] AC-06 / merge：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、未来实施授权及当前源 | 已建专属树 `lang-1028` 与 `plan-artifacts/plan-1028`，完成四件源/Spec SHA256 与漂移核查，冻结内层隔离策略与五组并集；AC-01 |
| T-02 / work | [x] T-01决定与可区分输入 | 先落 `OBS-01`/`OBS-02` 永久反例取得修前红证据，再于 `core/src/lib.rs` 抽出 `capture_scoped` RAII 守卫并取得修后绿；AC-02 |
| T-03 / work | [x] T-02 | `rustfmt --check` 零告警、无新增编译告警；五固定组并集 2336/2336 PASS；`SD-01` Spec 已于 worktree 准备并提交 `be7f1554d`；AC-02/03 |
| T-04 / review | [x] T-01..03完成、冻结提交 | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；绑定 `be7f1554d` 与五组并集 PASS；AC-04 |
| T-05 / merge | [x] T-04 pass | 已 rebase 至 `9a7674009`（`range-diff` 1:1 等价）、重跑 `ace-core`/`ace-native` `observation_scope_1028` 2+1 PASS、ff-only 合入 `7880f2a42`、核验 `closures-function-production.md` SHA256 `6ef4305d...` 并归档；AC-05 |
| T-06 / merge | [x] T-05 | 所有输出路径核查、摘要已折入并全删 `plan-artifacts/plan-1028`（`.retention/policy.json` 已更新，`holds: []`）与 `%TEMP%\plan1028-*`；`wt-guard.sh` clean 后已移除 `{auto-lang,auto-down}` 工作树、`plan-1028-dev` 分支及组目录；AC-06 |

同文件串行；跨计划调度见1024分配表。

## 9. 复审记录

- 2026-11-13 `stage: new | plan_id: PLAN-1028 | plan_revision: r1 | outcome: pass`（可审阅规划，非产品/独立实施review） `| next: 获实施授权后work`
- 2026-11-13 `stage: work | plan_id: PLAN-1028 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1028/auto-lang | branch: plan-1028-dev | base_commit: c2760324da25795594ab0cc01b581e3a306d2375 | code_commit: be7f1554d785f6a9a2ea5d9901fa8a344f1dcf17 | dependency_revisions: auto-down@58ea33f | task_ids: T-01..T-03 | evidence: OBS-01/OBS-02 pre-fix red (pref-red-core.log / pref-red-native.log) -> post-fix green + cargo t ace-source ace-native ace-av ace-semantics ace-a2x (2336/2336 PASS, gate_passed: True, source_state: 77046e8a31bf8270aa9f7d30b5b908cf0c11404570ff198b871ef8afca370f7e) | blockers: none | next_responsible_role: review`
- 2026-11-13 `stage: goal | plan_id: PLAN-1028 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1028/auto-lang | commit: be7f1554d785f6a9a2ea5d9901fa8a344f1dcf17 | next: review`
- 2026-11-13 `stage: review | review_scope: final | reviewer_identity/limitations: single-session /auto-plan-goal review (Plan §6 unsplit; reconstructed from frozen commit be7f1554d, git diff, test replay, and cargo-t results.json) | plan_id: PLAN-1028 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1028/auto-lang | branch: plan-1028-dev | reviewed_commit: be7f1554d785f6a9a2ea5d9901fa8a344f1dcf17 | base_commit: c2760324da25795594ab0cc01b581e3a306d2375 | dependency_revisions: auto-down@58ea33f | spec_inputs: docs/specs/auto-ac/closures-function-production.md@6ef4305d2f2f80ec5dec50eb3408751f36bab9e843f845d8dff392c3e9be4bab | acceptance_results: AC-01=pass, AC-02=pass (OBS-01/02 pre-fix red + post-fix green 3/3), AC-03=pass (5-group union 2336/2336 PASS), SD-01=pass | pending_closeout_gates: ff-only merge onto master, canonical Spec verify on master, Plan archive commit, path reconciliation & plan-artifacts/plan-1028 delete + policy.json refresh, wt-guard clean + worktree/branch removal, .last-batch-regression.json due check | findings: none | evidence: git show be7f1554d clean, rustfmt --check clean, zero new warnings, ace-core/ace-native observation_scope_1028 replay PASS, cargo-t-union-results.json gate_passed=True | next: merge`
- 2026-11-13 `stage: goal | plan_id: PLAN-1028 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1028/auto-lang | commit: be7f1554d785f6a9a2ea5d9901fa8a344f1dcf17 | next: merge`
- 2026-11-13 输出路径与暂存对账清单（`plan-artifacts/plan-1028` 共 6 个文件，合计 1,710,044 字节，已折叠摘要后准备于 T-06 全删，具名持有 `holds: []`）：
  - `pref-red-core.log` (`1713` B, SHA256 `7B4FA922F6C979E5080BE0BC16CE82886AB046EE6C90A88A7EB6FA3510F9539BB`) — 修前红证据（`obs_01` L59 / `obs_02` L221 `None`）
  - `pref-red-native.log` (`8535` B, SHA256 `8EFFC29388A1638064CC1F2B9D38657B2683A8226AFBF813876098EB8AC2A231`) — 修前红证据（`native` L60 `outer_a` `{}`）
  - `post-green-core.log` (`1809` B, SHA256 `A04FD70CDD567F0AB8A5C0D8921642BAEA32CD69805C226A8A97732D1C946AA5`) — `be7f1554d` `ace-core` 2/2 PASS
  - `post-green-native.log` (`5073` B, SHA256 `9FF330F7EE30E38C9FAE479B1E4FB973DA857CB937A6D27C49D00775F06E9854`) — `be7f1554d` `ace-native` 1/1 PASS
  - `cargo-t-union.log` (`274` B, SHA256 `95B35FE8EE778530219DA38443F9E9BF9CD9AE05127356E2FAF4AF6E27938628`) — 五固定组并集入口日志
  - `cargo-t-union-results.json` (`1692640` B, SHA256 `611A781805AB1B5AF71F9CE072C1C7D9662529C6DAE7858E52745920199BFA2E`) — 五固定组 2336/2336 PASS 完整结果
  - worktree 构建缓存 `D:/autostack/.wt/lang-1028/auto-lang/experimental/ace/target/`：不回收、随 worktree 移除销毁；外部生产产物核查：本计划仅修改 `experimental/ace/crates/core`、conformance 测试与 `docs/specs/auto-ac/`，未触及 `crates/auto-lang`、`auto-down`、`auto-ui`、`target/release/auto.exe` 或 web bundle；残留临时目录 `$env:TEMP\plan1028-nested-35156-1794574520545226500`（修前红 panic 遗留）待 T-06 一并清除。
- 2026-11-13 `stage: merge (archive checkpoint) | plan_id: PLAN-1028 | plan_revision: r1 | delivery_commit: 7880f2a425b535a9aee2ab97c43106c604de5842 | rebased_from: be7f1554d785f6a9a2ea5d9901fa8a344f1dcf17 | range_diff: 1:1_equal | master_delta_checked: c2760324d..9a7674009 (docs-only, zero_ace_code_change) | rebased_replay: ace-core observation_scope_1028 2/2 PASS + ace-native observation_scope_1028 1/1 PASS | canonical_spec: docs/specs/auto-ac/closures-function-production.md@6ef4305d2f2f80ec5dec50eb3408751f36bab9e843f845d8dff392c3e9be4bab | archive_path: docs/plans/archive/1028-ace-scoped-observation-restoration.md | pending: T-06 worktree/artifacts cleanup and batch regression due check`
- 2026-11-13 `stage: merge | plan_id: PLAN-1028 | plan_revision: r1 | outcome: pass | checkpoint: cleaned (6/6, AC-01..06 PASS) | delivery_commit: 7880f2a425b535a9aee2ab97c43106c604de5842 | canonical_spec: docs/specs/auto-ac/closures-function-production.md@6ef4305d2f2f80ec5dec50eb3408751f36bab9e843f845d8dff392c3e9be4bab | archive_path: docs/plans/archive/1028-ace-scoped-observation-restoration.md | cleanup_state: wt-guard clean on {auto-lang,auto-down}; worktrees, branch plan-1028-dev, group dir .wt/lang-1028, plan-artifacts/plan-1028 (6 files / 1,710,044 B), and %TEMP%/plan1028-* deleted; .retention/policy.json updated (scope-deleted-at-merge, holds=[]) | production_artifacts: scope limited to experimental/ace tests/core + docs/specs/auto-ac (target/release/auto.exe not present/unaffected) | batch_regression_due_check: due=true (PLAN-1025 landed after last_covered_plan_id=1015 @d1ee64099; receipt age <48h); execution_deferred_to_exclusive_window (concurrent cargo/cargo-nextest PIDs 14764/5936/20228/39332/53276/57668/59020 running on machine + concurrent dirty docs on master; run /auto-plan:regress when idle)`
- 2026-11-13 `stage: goal | plan_id: PLAN-1028 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | completion_kind: delivered | outcome: pass | delivery_commit: 7880f2a425b535a9aee2ab97c43106c604de5842 | archive_path: docs/plans/archive/1028-ace-scoped-observation-restoration.md | next: done`

## 10. 待澄清事项

不改变正式计量样本/预算，不由观察修复反推历史验收无效；内层是否累加如与已承诺接口冲突，先T-01记录决定。 T-01使真实反例/资源或IO故障可运行，未取得前不声称动态后果已证。等价实现不需重复审批；新支持/预算/公共合同变化先最小决定，不能用隐式门限或延期替原义务。不等待“新测试框架全部完成”。
