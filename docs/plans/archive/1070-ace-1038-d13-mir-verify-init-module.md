---
plan_id: PLAN-1070
status: drafting                  # drafting → executing → execution_done → reviewed → archived
feature_name: ACE MIR verifier init 分析私有模块边界（1038 D13 首片）
author: [lead]
created_at: 2026-11-17
updated_at: 2026-11-17

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [ace, auto-av]
current_step: 0
total_steps: 5
plan_revision: 1
depends_on: [PLAN-1038]
---

# [PLAN-1070] ACE MIR verifier init 分析私有模块边界（1038 D13 首片）

## 0. 变更摘要

执行已归档 [PLAN-1038](../plans/archive/1038-ace-reuse-candidates-and-cost-decisions.md) §9 T-02 的 **D13（次序列）**：只做一个 crate 私有模块边界——把 `av-core/src/mir_verify.rs` 的 initialization-analysis family（`check_init`、`init_analysis_bound`、位集 helpers）移入新私有子模块 `mir_verify/init.rs`，父 `mir_verify/mod.rs` 保留凭证（`CheckedMir`）、`run()`、类型/借用/CFG 规则与 `slot_ty_of` 等跨族 helper。public API、minting 边界与 `MirVerifyError` 不变，无运行期语义变化。

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

- [ ] AC-01 / work：`mir_verify` 成为目录模块，init 族按 §5.1 移入 `init.rs`，父文件仅两处调用点变化；`mod init` 私有、无新公共 API、`CheckedMir` 构造仍唯一在父模块。
- [ ] AC-02 / work：`cargo t ace-av` 全绿（INIT-01..05 原 case 全过），`cargo check -p ace-av-core` 0 error；diff 仅移动（无算法/预算/错误码改动）。
- [ ] AC-03 / work：无新增 warning；`rustfmt --check` 改动文件通过；未提交模板/探针；公共 API 面（`cargo doc`/符号）无新增。
- [ ] AC-04 / review：独立 context 复核 AC-01..03、迁移完整性（无函数丢失/重复）与行为等价；结论绑定提交。
- [ ] AC-05 / merge：ff 合入、归档、清树/分支与输出对账。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [ ] 1038 D13 合同与当前源 | 重核 §4 身份与 §5.1 行号/使用点；建 plan-artifacts scope。AC-01 |
| T-02 / work | [ ] T-01 | `git mv` 成目录模块；抽 `init.rs`；改两调用点；`cargo check`/`cargo t ace-av`；diff 审查。AC-01/02 |
| T-03 / work | [ ] T-02 | 健康检查（warning/rustfmt/公共 API 面）；记录行数前后与 diff stat。AC-03 |
| T-04 / review | [ ] T-01..03 | 独立 context 复核迁移完整性与等价性；失败回 work。AC-04 |
| T-05 / merge | [ ] T-04 pass | 冻结门复用、ff 合入、归档、guard+清树、registry。AC-05 |

## 9. 复审记录

2026-11-17 stage:new / PLAN-1070 r1 / outcome:pass（可执行规划，非代码/运行 PASS） / next: work。边界与 oracle 取自已归档 1038 §9 T-02 D13；迁移清单按当前源行号核到调用点；SD-00 无规范增量；未建树、未改实现/测试/Spec、未运行测试。

## 10. 待澄清事项

- 若 `init.rs` 对父私有的访问需要比 `pub(super)` 更宽（例如借用检查），只允许 `pub(crate)` 且不得成为对外 API；在 §9 记录。
- 本包不含另外五个热点与 Linux 腿。
