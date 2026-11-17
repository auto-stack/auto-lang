---
plan_id: PLAN-1070
status: archived                  # drafting → executing → execution_done → reviewed → archived
feature_name: ACE MIR verifier init 分析私有模块边界（1038 D13 首片）
author: [lead]
created_at: 2026-11-17
updated_at: 2026-11-17

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [ace, auto-av]
current_step: 5
total_steps: 5
plan_revision: 1
depends_on: [PLAN-1038]
---

# [PLAN-1070] ACE MIR verifier init 分析私有模块边界（1038 D13 首片）

## 0. 变更摘要

执行已归档 [PLAN-1038](1038-ace-reuse-candidates-and-cost-decisions.md) §9 T-02 的 **D13（次序列）**：只做一个 crate 私有模块边界——把 `av-core/src/mir_verify.rs` 的 initialization-analysis family（`check_init`、`init_analysis_bound`、位集 helpers）移入新私有子模块 `mir_verify/init.rs`，父 `mir_verify/mod.rs` 保留凭证（`CheckedMir`）、`run()`、类型/借用/CFG 规则与 `slot_ty_of` 等跨族 helper。public API、minting 边界与 `MirVerifyError` 不变，无运行期语义变化。

其余五个热点（resolve/adapter/verify/link/AV execute/project）**不改**；不拆 crate、不改规则、不改预算。

## 1. 目标

- **G1**：init 分析族成为独立私有维护单元（约 300 行），父文件按规则族收敛；`mod init;` 为私有（不导出新公共 API）。
- **G2**：`CheckedMir` 仍只由父模块构造；`verify_mir`/`validate_mir*` 公共签名与行为逐字节等价（同一 verifier、同一 analysis 预算计算）。
- **G3**：`cargo t ace-av` 全绿（含 `mir_admission_*` init 组：未初始化读、分支合流、循环 init、dense analysis 预拒），无新增 warning、rustfmt clean、diff 仅移动。

**非目标**：不改 init 算法/预算/拒绝码；不移动类型/借用/CFG/span 规则；不处理另外五个热点；不拆 crate；不动其他计划 WIP。

## 2. 架构方案

`git mv av-core/src/mir_verify.rs av-core/src/mir_verify/mod.rs`，新增 `mir_verify/init.rs`：

| 项 | 位置（迁移后） |
|---|---|
| `CheckedMir`/`VerifyStats`/`MirVerifyError`、`verify_mir*`/`validate_mir*`、`run()`、`check_function`、CFG/类型/借用/捕获规则、span 规则、`slot_ty_of` | `mir_verify/mod.rs`（不变，除两处调用点） |
| `check_init`（由 `&mut Verifier` 接收者改为 `pub(super) fn check_init(v: &mut Verifier, …)`）、`init_analysis_bound`（`pub(super)`）、`words_for`/`set_bit`/`bit_is_set`/`union`/`union_into`/`intersect` | `mir_verify/init.rs`（私有 except 两 `pub(super)`） |

调用点改动仅两处：父 `check_function` 内 `self.check_init(f, &bases)` → `init::check_init(self, f, &bases)`；`init_analysis_bound(f)` → `init::init_analysis_bound(f)`。`check_init` 内 `self.tick(...)`/`self.span_of(...)`/`self.module` 改 `v.…`；其余语义逐行不动（含 worklist、RPO、reachability、`def_slot`/`use_slots`、预算公式）。

## 3. 技术栈

Rust（av-core，ACE 独立 workspace）、`cargo t ace-av`。唯一树 `D:/autostack/.wt/lang-1070/auto-lang`，分支 `plan-1070-dev`；材料 `D:/autostack/plan-artifacts/plan-1070/`（owner 1070，用途：迁移前后 diff/门禁摘录，在途）。

## 4. 需求分析与背景调查

**授权**：用户“继续，直到 1038 全部完成”；1038 已归档并把 D13 定为“单一 crate 私有边界、次序列”，边界与 oracle 见 1038 §9 T-02 D13 行。本包是该决定的直接实施，不扩范围。

**当前源身份（开工复核）**：

| 路径 | 行数 | SHA256 |
|---|---|---|
| experimental/ace/crates/av-core/src/mir_verify.rs | 5463 | c2628e251f772961d953e17698701534702f6858893eee36d7414118080a43af |
| experimental/ace/crates/av-core/src/lib.rs | 76 | （开工记） |

**已核事实**：`check_init` 唯一调用点在其父 `check_function`（4406）；`init_analysis_bound` 唯一父调用点 4155；`check_init` 内不含 `slot_ty_of`，只用 `def_slot`/`use_slots`/`self.tick`/`self.span_of`/`self.module`；位集 helpers 与 `words_for` 的使用点全部落在 init 区（4490/4559/4618 等），测试模块不引用它们；`CheckedMir` 构造在父模块（85–131）。行数/churn 证据见 1038 §9 T-02 D13（resolve 87 / adapter 52 / verify 39 / mir_verify 31 次近 400 提交）。

## 5. 详细设计

### 5.1 迁移清单

- **移入 `init.rs`**：`check_init`（父 4410–4645 原文）、`init_analysis_bound`（4885–4916）、`words_for`（4918–4920）、`set_bit`（4939–4941）、`bit_is_set`（4943–4947）、`union`（4949–4951）、`union_into`（4953–4957）、`intersect`（4959–4963）。
- **保留在父 `mod.rs`**：`slot_ty_of`（4923–4937）、`MirInstr::def_slot`/`use_slots`、`span_of`/`span_of_fn`、其余全部规则与凭证构造。
- **可见性**：`init.rs` 内两入口 `pub(super)`；父 `mod init;` 为私有；模块文档说明这是 init dataflow 单元，不是新公共 API。
- **不变式**：`verify_mir` 的 stage 顺序、预算（`init_analysis_bound` 公式逐字不动）、trap/拒绝码、`VerifyStats` 字段与值不变；`CheckedMir` 构造点仍唯一在父模块。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-00 | 无（内部私有模块边界） | 无 | 不改 canonical Specs：MIR verify 的规则/凭证/预算与公共 API 不变，仅同 crate 私有文件组织变化 | 内部可维护性改造，无对外行为 | AC-01..03 |

## 6. 测试设计

| Case ID | AC/owner | 输入/状态 | 入口 | 独立 oracle | 状态 |
|---|---|---|---|---|---|
| INIT-01 | AC-02 / T-02 | 未初始化读 | av-core `mir_admission_uninitialized_read_rejected` | 原错误码/消息不变 | 原 case 复用 |
| INIT-02 | AC-02 / T-02 | 分支合流两臂 | `mir_admission_branch_merge_needs_both_arms` | 同上 | 原 case 复用 |
| INIT-03 | AC-02 / T-02 | 循环 init 不外逃 | `mir_admission_loop_init_does_not_escape` | 同上 | 原 case 复用 |
| INIT-04 | AC-02 / T-02 | dense analysis 预拒 | `mir_verify_dense_analysis_memory_pre_rejected` | 资源拒绝而非 Invalid | 原 case 复用 |
| INIT-05 | AC-02 / T-02 | 合法入口通过 + analysis bytes 记录 | `mir_admission_minimal_entry_passes`、`mir_verify_analysis_bytes_recorded_on_credential` | 原断言 | 原 case 复用 |

门禁：`cargo t ace-av`（Windows；Linux 腿按 2026-11-14 裁定排除）；`cargo check -p ace-av-core`；`rustfmt --check` 改动文件；不跑 ace-perf/`cargo t all`。角色：work 本会话、review 独立 subagent、merge 同会话。

## 7. 验收标准

- [x] AC-01 / work：`mir_verify` 成为目录模块，init 族按 §5.1 移入 `init.rs`，父文件仅两处调用点变化；`mod init` 私有、无新公共 API、`CheckedMir` 构造仍唯一在父模块。
- [x] AC-02 / work：`cargo t ace-av` 全绿（INIT-01..05 原 case 全过），`cargo check -p ace-av-core` 0 error；diff 仅移动（无算法/预算/错误码改动）。
- [x] AC-03 / work：无新增 warning；`rustfmt --check` 改动文件通过；未提交模板/探针；公共 API 面（`cargo doc`/符号）无新增。
- [x] AC-04 / review：独立 context 复核 AC-01..03、迁移完整性（无函数丢失/重复）与行为等价；结论绑定提交。
- [x] AC-05 / merge：ff 合入、归档、清树/分支与输出对账。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1038 D13 合同与当前源 | 重核 §4 身份与 §5.1 行号/使用点；建 plan-artifacts scope。AC-01 |
| T-02 / work | [x] T-01 | `git mv` 成目录模块；抽 `init.rs`；改两调用点；`cargo check`/`cargo t ace-av`；diff 审查。AC-01/02 |
| T-03 / work | [x] T-02 | 健康检查（warning/rustfmt/公共 API 面）；记录行数前后与 diff stat。AC-03 |
| T-04 / review | [x] T-01..03 | 独立 context 复核迁移完整性与等价性；失败回 work。AC-04 |
| T-05 / merge | [x] T-04 pass | 冻结门复用、ff 合入、归档、guard+清树、registry。AC-05 |

## 9. 复审记录

2026-11-17 stage:new / PLAN-1070 r1 / outcome:pass（可执行规划，非代码/运行 PASS） / next: work。边界与 oracle 取自已归档 1038 §9 T-02 D13；迁移清单按当前源行号核到调用点；SD-00 无规范增量；未建树、未改实现/测试/Spec、未运行测试。

### T-01/T-02 迁移实施（2026-11-17，work）

- 树：`D:/autostack/.wt/lang-1070/auto-lang`（branch `plan-1070-dev`，base `19281566f`）；材料 `D:/autostack/plan-artifacts/plan-1070/`。
- 源身份（开工）：`mir_verify.rs` 5463 行 / `c2628e25…`（与 1038 T-01 一致，无漂移）；`av-core/src/lib.rs` 76 行。
- 迁移：`git mv mir_verify.rs mir_verify/mod.rs` + 新增 `mir_verify/init.rs`（314 行）：`check_init`（改为 `pub(super) fn check_init(v: &mut Verifier, f, bases)`）、`init_analysis_bound`（`pub(super)`）、`words_for`/`set_bit`/`bit_is_set`/`union`/`union_into`/`intersect`；父文件保留 `slot_ty_of`、`def_slot`/`use_slots`、凭证/证据/类型/借用/CFG/span 规则，只改两处调用点（`init::check_init(self, …)`、`init::init_analysis_bound(f)`）并声明私有 `mod init;`。
- 等价性自检（脚本对比 base 与迁移后）：命名函数集合 66（父 58 + 子 8），**无丢失/无新增/无重复**；8 个被移动函数的 body 归一化（`self.`→`v.`）后**逐行相等**（check_init 仅 3 处接收者改写 `v.tick`/`v.module`/`v.span_of`，其余体逐行相同；init_analysis_bound 与 6 个位集 helper 逐字相同）。
- `cargo check -p ace-av-core` exit 0；`rustfmt --check` 两文件通过；无新增 warning（`VecDeque` 导入随族移入 init.rs，父文件移除）。
- diff stat：`mod.rs` −299/+4；`init.rs` 新增 314 行；无算法/预算/错误码文本改动。
### T-03 门禁与健康（2026-11-17，work）

- 首次全组 `cargo t ace-av`：1354 tests，2 failed（`kv_zip_1064::v03_kv_wire_oracle_and_reopen_av`、`v03_zip_negative_corrupt_bomb_and_preserve_av`），两例均为 `Resource{kind: ExecutionWall}`（"deep clone/platform call exceeded the execution wall budget"），失败点 `kv_zip_1064.rs:88`；与本包纯移动 diff 无因果关系（同族其余 1352 通过）。
- 归因：过滤重放 `cargo t ace-av --test-name kv_zip_1064` **PASS**（机器空闲后）；随后**全组重跑 PASS**：`executed=1354`、`exec_wall=75.2s`、`not_run=[]`、`errors=[]`（`run-jf82htd9`，`source_state=5a383f5cee62040c3929ef8fb7023f3fa33f97d51207731c8fb1250893e06f35`）。按环境/负载归因记录，不作为代码红。
- 健康：`cargo check -p ace-av-core` exit 0；无新增 warning（`VecDeque` 随族移入 init.rs、父文件删该导入）；`rustfmt --check` 两文件通过；INIT-01..05 全在组内通过；`CheckedMir` 构造点仍唯一在父 `mod.rs`，`mod init;` 私有。
- 提交：`e6a0d56be`（`mod.rs` +4/−299、`init.rs` +314，git 识别 95% rename）。
### T-04 独立复审（2026-11-17，review，scope: final）

`stage: review | review_scope: final | reviewer_identity: 独立 context subagent reviewer-1070（未采信 work 摘要；自读 diff、重算函数/体等价、重跑全组门） | plan_id: PLAN-1070 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1070/auto-lang | branch: plan-1070-dev | reviewed_commit: e6a0d56be | base_commit: 19281566f | acceptance_results: AC-01 pass / AC-02 pass / AC-03 pass / next: merge`

复核要点：

- AC-01：`mod.rs:24` 为私有 `mod init;`；`init.rs:16` `pub(super) fn check_init(`、`:253` `pub(super) fn init_analysis_bound(`，6 个 helper 私有；全 crate 仅两处调用（`mod.rs:4156`、`:4407`）；`CheckedMir {` 仅一处 mint（`mod.rs:119`）；`lib.rs:68` 公共面未动；提交只含 2 文件（95% rename）。
- AC-02：base 与迁移后函数名集合 66（58+8），无丢失/新增/重复；`init_analysis_bound` 与 6 个 helper **逐字相同**；`check_init` 仅 3 处接收者改名（`tick`/`module`/`span_of`）；预算 saturating 链逐字保留（`init.rs:274–280`）、拒绝码 `mir.uninitialized-read`（`init.rs:213`）、防御步数门（`init.rs:246`）不变；父保留 `slot_ty_of`、`def_slot`/`use_slots`、span helper。reviewer 自跑 `cargo t ace-av` **PASS**：`executed=1354`、0 失败、`not_run=[]`、`errors=[]`、`wall=92.16s`、**0 flake**（`run-jkgshmwi`，`source_state=264fb7240572869f93c3bfa56482394c403fefd26176c7c25bd58f296c081048`，绑定冻结提交）；INIT-01..05 全绿。
- AC-03：`rustfmt --check` 两文件 exit 0；`cargo check -p ace-av-core` exit 0；9 个既有 warning 全在未触碰的 Linux-only bin（`mn-linux-av-runner.rs:26–34`），**无新增**；无 probe/模板入库；公共 API 无增长。

发现与处置：

- **F-01（低，SD-00 溯源）**：9 处 canonical Spec 仍以 `mir_verify.rs` 文件形式指向该模块（`auto-mir/project.md:7`、`core-i32.md:3`、`dynamic-cast-profile.md:3`、`closure-capture-profile.md:3`、`function-incremental.md:4`、`collection-place-profile.md:3`、`ace-migration/m2-control-effects.md:5`、`m2-numbers.md:6`、`m2-stdlib-network.md:15`）。规范内容（规则/预算/拒绝码/凭证）不变，SD-00 成立；路径指针作为**有界文档后续**登记到 `KNOWN-DEBT-AND-RISKS.md`（不阻塞本包）。
- **F-02（低，证据数字）**：§9 原记 `init.rs` 311 行/70 函数/227 行体，已按 reviewer 复算更正为 314 行、66 个命名函数（58+8）、check_init 仅 3 处接收者改写。
- **F-03（限制）**：首次全组的两个 `ExecutionWall` flake 原始日志未保留（scope 未落盘），仅叙述；归因依据 `MAX_AV_WALL=2s`、`kv_zip_1064.rs:88` 的 `other => panic!` 臂、kv_zip 为组内最长用例（29–61s，并行度 20）成立，reviewer 独立全组 0 flake。
- **F-04（信息）**：work 侧 `source_state=5a383f5c…` 早于提交；以 reviewer 在冻结提交上的 `264fb724…` 作为 review-bound 证据（上一行）。
### T-05 合并、归档与清理（2026-11-17，merge）

`stage: merge | plan_id: PLAN-1070 | plan_revision: r1 | outcome: pass | completion_kind: delivered | review: final pass（§9 T-04） | delivery_commit: ff1433659（`git merge --ff-only plan-1070-dev`，无 merge commit） | canonical: SD-00 无规范增量（私有模块组织；行为/接口/身份不变） | archive: docs/plans/archive/1070-ace-1038-d13-mir-verify-init-module.md（status: archived） | cleanup: 实施树/分支/组目录删除，无 plan-artifacts 持有 | next: done`

| checkpoint | evidence |
|---|---|
| prepared | reviewed r1 @e6a0d56be；diff 2 文件（95% rename + 新 `init.rs`）；SD-00 无增量 |
| landed | rebase 到 `9d1774f46` 后 `git merge --ff-only plan-1070-dev` → master `ff1433659`；reviewed→delivered 映射 `e6a0d56be→ff1433659`，`git range-diff` 1/1 patch 相等；该窗口内 master 含 ACE 变更（PLAN-981 B-W1 两次 fold），故门禁在 rebased 树上重跑而非复用旧证据 |
| canonical_verified | 交付提交内 `docs/specs/` 零改动；SD-00（私有目录模块，无规则/预算/拒绝码/凭证变化）成立。review F-01 指出的 9 处 Spec 路径指针（`mir_verify.rs`→目录模块）登记为有界文档债务 `D-1070-01`，不在本包改 canonical 文本 |
| archived | 计划已移至 `docs/plans/archive/` 并置 `status: archived`；归档位置链接可解析；`KNOWN-DEBT-AND-RISKS.md` F13 标为已修@1070 |
| cleaned | `wt-guard.ps1` clean 后删除 `D:/autostack/.wt/lang-1070/auto-lang`、分支 `plan-1070-dev`、空组目录与空 `plan-artifacts/plan-1070`；registry 增补 1070 关闭条目（无持有） |

门禁与观察：

- **冻结树门禁**：`cargo t ace-av` 在 landed 树 **PASS**：`executed=1360`、`exec_wall=85.3s`、0 errors（`run-xugkxj9h`，`source_state=e6fd8ec1888d6f18e3d6466a8d242ec6391ee3c007c3c9aa7d071e1d987138c1`，`config=a3c63e57…`）；INIT-01..05 全在组内绿。
- **环境红记录（不充 PASS）**：rebased 树上前两次整组（`run-cg6cq93o`）与首次（`run-crr6_lx2`）各 2 例 `kv_zip_1064` `ExecutionWall`（31.1s/35.0s vs 2s 预算；另一例 deep clone）；隔离重放 `cargo t ace-av --test-name kv_zip_1064` 两次绿；第三次整组绿。与 KNOWN-DEBT **D-1037-02**（同一用例 2s wall 余量过窄，已登记环境归因）一致；reviewer 在 `e6a0d56be` 上的独立整组亦 0 flake。
- **部署新鲜度**：无生产产物受影响 → 无需 rebuild/restart。
- **批量回归到期**：`.last-batch-regression.json` last_covered 1064 @2026-11-16T10:04Z；1070%5=0 → **按规则本包落地后应触发一次 `cargo t all` 批量回归**（主检出单实例；归 `/auto-plan:regress` 后续执行，不阻塞本包归档）。
## 10. 待澄清事项

- 若 `init.rs` 对父私有的访问需要比 `pub(super)` 更宽（例如借用检查），只允许 `pub(crate)` 且不得成为对外 API；在 §9 记录。
- 本包不含另外五个热点与 Linux 腿。





