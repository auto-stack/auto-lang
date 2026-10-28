---
plan_id: PLAN-872
status: drafting
feature_name: 内建语义双轨裁定与收敛（Str/Math/Time/Reply/nv_to_value）
author: [DeepSeek Harness session]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 2
current_step: 0
total_steps: 10
supersedes_spec_components: [docs/specs/stdlib/project.md, docs/specs/auto-lang/vm/overview.md]
new_spec_components: []
touched_goals: []              # 有意留空：收敛实现、不改 GOAL 定义；GOAL-003/007 的语义一致性是本计划的动机而非目标变更
affects: [language-sdk, stdlib, auto-ui, auto-man, test-infrastructure]
depends_on: [PLAN-867]
---

# [PLAN-872] 内建语义双轨裁定与收敛（Str/Math/Time/Reply/nv_to_value）

> **整 Plan 启动门 G872 = U867 且 U869**：867 交付三仓完整档与 ID 闭合（验收基础），869 的减法批次不得先动这 6 族（869 §1 非目标已声明）。门未齐只做只读核查与规划簿记。
> 本计划处理 [PLAN-844](archive/844-cross-repository-dedup-and-net-slimming.md) §2 保留表里明确"超出减法授权"的 6 族；**它们是语义问题，不是重复代码问题**，因此不能沿用"删重复"的判据。

## 0. 变更摘要

844 在去重时冻结了 6 族"看起来重复、实际语义不同"的内建实现，并把它们排除在减法之外：`str.len`（字符 vs 字节）、`math` 20+ 族（0 兜底 vs Err）、`to_int`/`parse_int`、`time_now`（`unwrap_or(0)` vs `unwrap`）、HTTP error-reply 采样族 ×10、`nv_to_value` 双签名。它们的真实差别常常**不在函数体，而在注册/绑定序**：同名 builtin 由"纯 shim（catalog/inventory）"与"native 覆盖块"共同决定最终生效实现（844 D-4 已实证：纯版 `contains/starts_with/ends_with` 恒被覆盖，删纯版后须显式补名字注册）。

因此本计划的第一步不是改代码，而是**冻结一张真实的 builtin 绑定表**（名字 → 注册面 → 直接实现 → 生效实现 → 语义 → 消费者 → 钉住它的测试），再按用户 2026-10-28 的裁定对六族**统一按 DISTINGUISH 处置**（保留两侧、把名字/文档/测试讲清楚，零行为变更）；MERGE／RETIRE 均不在本轮实施面（RETIRE 仅在某侧完全无消费者时另行签核）。

对比例的已见实例：`Str.len` 纯 shim 的文档写 "length in bytes"、实现是 `s.len()`（字节），而同文件 `Str.byte_len` 的注释称 `str.len()` 是 PLAN-055 的**字符数**语义——三者至少有一个是错的，DISTINGUISH 的活就是把"谁在运行时生效"实测钉死，再把名字/文档/测试对齐。

## 1. 目标

- **G1 绑定表可复算**：6 族涉及的全部 builtin 名字，按真实注册/覆盖顺序列出"生效实现"，并可用最小程序实测覆核（不是读码猜测）。
- **G2 每族有裁定（已定）**：用户 2026-10-28 裁定 **六族全部按 DISTINGUISH 落地**（保留两侧，把名字/文档/测试讲清楚，**零行为变更**）；MERGE 未获授权，本计划不得实施任何用户可观察的行为变更；RETIRE 仅在 T-02 证明某侧完全无消费者时提出，并需单独签核。
- **G3 无静默漂移**：处置完成后，名字、文档注释、规范页、测试四者对同一语义一致；`str.len` 这类"名字/文档/实现三方矛盾"不得残留。
- **G4 不破坏既有承诺**：`plan755` 的 HTTP 契约 body、JS/Vue 的 `.length` 语义、a2r 生成代码的调用形态保持；改变任一承诺须显式记录并取得用户同意。
- **G5 门禁联动**：867 的三仓完整档复跑通过；被合并/退役实现的测试按 867 账本落 RETIRED 或 RENAMED 有据。

**非目标**：不做 869 的 crate 清单与减法；不重写 stdlib/VM 架构；不动 `lib-legacy`、aavm、`type_registry` 等 844 保留有证项；不新增 backend 或新内建名字（除 DISINGUISH 需要的新名字）；不把 error-reply 采样族当成性能优化去重写（仅在同 body 字节前提下才允许抽 helper）。

## 2. 架构方案

1. **绑定表（T-01）**：从三处真源读：① `#[auto_macros::rust_fn("Name")]` 属性注册面（纯 shim 轨道，如 `ffi/stdlib/*.rs`）；② native 覆盖块/名字表（`vm/native.rs` 与 `vm/native/*.rs`）；③ 运行时 `production()` 绑定序（catalog → inventory → nv 覆盖）。产出 `name ⇥ 生效实现 ⇥ 语义 ⇥ 证据`，并用最小 Auto 程序实测对照（VM 侧"字符 vs 字节"等可直接观测）。
2. **裁定（T-02）**：六族已裁定为 **DISTINGUISH**；T-02 仍须逐族出证据（消费者/公开承诺/语义差异）并确认"不存在完全无消费者的单侧"（若存在则提 RETIRE 候选、单独签核）。**MERGE 未授权，不得作为实现路径**（未来若要合并，另开 revision 并取得用户裁定）。
3. **实施（T-04）**：按 DISTINGUISH 改文档/补名字/补测试，**不改运行时行为**；MERGE 未授权，不保留迁移窗口（未来若要合并另开 revision）。
4. **收据（T-05/T-06）**：行为对照收据（同程序两侧输出）、契约测试（plan755）复跑、867 门复跑、规范页更新。

判定原则：

- **以运行时实测为准**，不以文本相似或代码注释为准（844 的教训）。
- MERGE 的成立条件（"两侧语义可被同一实现满足，且没有消费者依赖被放弃的一侧"）本轮未被认定成立；因此六族一律 DISTINGUISH。若消费者分属不同语义（JS `.length` 字符 vs 文件偏移字节），DISTINGUISH 是**唯一正确**处置。
- 任何改变用户可观察行为的处置都必须有用户裁定记录，不能以"整理代码"名义静默改语义。

## 3. 技术栈

Rust 实现面（`crates/auto-lang-core/src/vm/{native,ffi/stdlib}`）、`auto_macros::rust_fn` 注册机制与 `production()` 绑定序、Auto 最小程序（VM 侧行为对照）、nextest（定向 + 867 三仓完整档）、`plan755` HTTP 契约测试、`docs/specs` 规范页。

## 4. 需求分析与背景调查

### 4.1 授权与轮次

- 用户 2026-10-28 要求把"867 之后的后续计划"写成文档；本计划是其中一份（语义双轨）。
- 本轮只写合同：不建树、不改实现、不跑档、不改 canonical Specs。
- **实施前不需要再裁定 MERGE**：用户 2026-10-28 已裁定六族全部 DISTINGUISH；若实施中发现某侧完全无消费者，才需就 RETIRE 单独签核。

### 4.2 已观察事实（2026-10-28）

| # | 事实 | 证据 |
|---|---|---|
| F-1 | 844 冻结 6 族为"语义不同/超出减法授权"：`str.len` 字符/字节、`math` 双轨、`to_int`/`parse_int`、`time_now`、error-reply ×10、`nv_to_value` | [844 归档 §2 保留表/§6 未知决策 4](archive/844-cross-repository-dedup-and-net-slimming.md) |
| F-2 | 同名 builtin 的生效实现由绑定序决定：catalog → inventory → nv 覆盖；纯版可能恒被覆盖 | 844 D-4 证据（`strings_json.rs` 纯版 3 函数删除 + merge-gate 显式名字注册补回，提交 `5d03c980b`） |
| F-3 | **名字/文档/实现三方矛盾**：`Str.len` 纯 shim 文档写 "length in bytes"、实现 `s.len()`（字节）；同文件 `Str.byte_len`（PLAN-781）注释称 `str.len()` 是 PLAN-055 字符数语义 | `crates/auto-lang-core/src/vm/ffi/stdlib/strings_json.rs:5-8`、`:26-32` |
| F-4 | `Math.*` 28 个注册在 `ffi/stdlib/math.rs`；nv 轨道另有同名覆盖条目，错误语义不同（0 兜底 vs Err） | `ffi/stdlib/math.rs`；844 §2（`native.rs:1713-1747` 对 `math.rs:5-167`，行号为 844 T-01 时点，T-01 需复核） |
| F-5 | `Time.now/now_ms/now_sec/sleep_ms` 注册在 `ffi/stdlib/sys_env.rs:99-123`；nv 侧另有实现，失败处理不同（`unwrap_or(0)` vs `unwrap`） | `sys_env.rs`；844 §2（`native.rs:1834-1863`） |
| F-6 | `nv_to_value` 双版签名不同（`native.rs:2248` vs `interop.rs:128`），跨 FFI 边界 | 844 §2 |
| F-7 | error-reply 采样族 10 处（`server_dispatch.rs` ×5、`server_argbind.rs` ×2 段、`server_scope.rs` ×2），同构非逐字，body 受 `plan755` HTTP 契约钉住 | 844 §2；`plan755_http_contract_tests` |
| F-8 | 867（进行中）将交付三仓完整档 + ID 闭合 + 档位下限守卫；869 已声明这 6 族为其非目标 | [867](867-split-equivalence-test-closure.md)、[869](869-crate-inventory-and-subtraction-batch-2.md) |

### 4.3 规范依据

- [stdlib/project.md](../specs/stdlib/project.md)：内建名字与语义的现行登记面（本计划主要 SD 落点）。
- [auto-lang/vm/overview.md](../specs/auto-lang/vm/overview.md)：VM 内建注册与绑定序。
- 844 收据与 [net-slimming.md](../specs/repository-boundaries/net-slimming.md)：保留有证清单（本计划不得越权动）。
- `plan755` HTTP 契约测试：error-reply body 的既有钉定。

### 4.4 假设与不声称

- **A1**：绑定表可从属性注册 + 覆盖块静态导出，并由最小程序实测覆核；若存在动态注册路径（宏/脚本生成），T-01 必须补查并在表内标注。
- **A2**：行号引用（F-4/F-5/F-6）来自 844 T-01 时点，经过 844 的删除后可能漂移，T-01 以文件+名字为准重新定位。
- **不声称**：本轮未读完全部 6 族实现细节、未做实测对照；六族处置已裁定为 DISTINGUISH，因此不需要再判"是否该 MERGE"。F-3 的三方矛盾是**已读到的具体事实**，但"谁在运行时生效"仍需 T-01 实测。

## 5. 详细设计

### 5.1 builtin 绑定表（T-01，关键产物）

- 范围名字集（初值，T-01 以实际绑定结果补全）：`Str.len`、`Str.byte_len`、`Str.to_int`、`Str.parse_int`、`Math.*`（28+）、`Time.now`/`now_ms`/`now_sec`、`nv_to_value` 的两个调用面、error-reply 相关 `ApiReply::Full` 构造点。
- 每行字段：`name | 注册面（catalog/inventory/nv/动态）| 直接实现（file:line）| 生效实现 | 语义（字符/字节/Err/0 兜底/unwrap…）| 消费者（VM 程序/a2r 生成/vue 生成/Rust merged/内部）| 钉住测试 | 备注`。
- 覆核方式：每族 ≥1 个最小 Auto 程序，实际观测（例：CJK 字符串取 `Str.len` 与 `Str.byte_len`；`Math.sqrt(-1)` 的返回/错误；`Time.now` 在不可用环境下的行为——若不可构造则显式记 pending）。
- 产物：`docs/plans/attachments/872/binding-table.md`（md 摘要 + 逐族证据）+ 机器可读 `scripts/builtin-binding-table.json`（含生成命令与注册面指纹）。

### 5.2 逐族裁定（T-02）

| 族 | 现状（844 判定） | 处置（已裁定 2026-10-28） | 落地要点 |
|---|---|---|---|
| `str.len` / `str.byte_len` | 两个**活名字**（字符 vs 字节） | **DISTINGUISH** | JS `.length` 语义与 PLAN-055/781 消费者各自保留；`str.len` 文档改成与实际生效一致 |
| `math` 20+ 族 | 机制性双轨（tag 感知 pop vs Err） | **DISTINGUISH** | 两侧错误契约各自文档化并加区分性测试；合并留待未来 revision |
| `Str.to_int` / `parse_int` | 双版 | **DISTINGUISH** | 解析失败语义分别写明 |
| `Time.now` 等 | 失败处理不同（0 vs unwrap） | **DISTINGUISH** | 不可用环境下的行为写清；不改失败可见性 |
| error-reply ×10 | 同构样板，非逐字；body 被契约钉住 | **DISTINGUISH**（可选抽 helper，仅当 body 字节不变且净删>0） | `plan755` 契约、净收益 |
| `nv_to_value` | 双签名跨 FFI | **DISTINGUISH** | 跨边界签名差异保留并注明 |

**已裁定：六族全部 DISTINGUISH**（2026-10-28）。原判据（"只有两侧能被同一实现满足且无消费者依赖被放弃一侧才 MERGE"）本轮未被认定成立，因此 MERGE 不在实施面；每族的处置是"补名字（若缺）+ 改文档 + 补区分性测试"，不改变运行时行为。

### 5.3 实施与兼容（T-03/T-04）

- MERGE：**本轮未授权，不实施**（保留此条仅作未来 revision 的接口说明）。
- DISTINGUISH：补名字（若缺）、补语义文档、补区分性测试（两侧各 ≥1 例，含边界：CJK、空串、负数、Unicode 数字、不可用时钟）。
- error-reply：若抽 helper，必须证明 10 处 body 字节级不变（`plan755` 全绿），否则不做。

### 5.4 执行合同（work/review/merge 共用）

| 字段 | 内容 |
|---|---|
| 规则与边界 | G1–G5；六族 DISTINGUISH、**零行为变更**；不改 844 保留项；MERGE 未授权 |
| 实际路径 | 属性注册 + 覆盖块 + `production()` → 绑定表 → 最小程序实测 → 逐族裁定 → 实现/文档/测试 → plan755 + 867 门 |
| 所有权与状态 | 绑定表为验收输入（改需原因+哈希）；名字 owner=语言仓；a2r/vue 生成器调用面随名字变更同步 |
| 区分性用例 | V-01..V-06：绑定表复算、实测覆核、**行为零变更对照**、DISTINGUISH 文档一致、契约 body 不变、867 门复跑 |
| Oracle 与关闭 | 每 AC 绑命令与期望输出（含两侧程序输出的字节对照）；关闭需原始日志 + 哈希 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/stdlib/project.md | 内建名字语义从"分散注释"→ 单一 `name → 语义 → 生效实现` 矩阵，含 `str.len`/`str.byte_len`、`math` 错误契约、`time` 失败行为 | 现有三方矛盾（F-3） | AC-02, AC-03 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md | 明确绑定序（catalog→inventory→nv 覆盖）与"同名只允许一个生效实现、纯 shim 不得静默被覆盖"的规则 | 844 D-4 暴露的机制 | AC-01, AC-05 |

## 6. 测试设计

`owner_stage` 均为 `work`；本轮未执行。

| Case ID | AC / invariant | 输入或 setup | 真实入口 | 可观察期望 | 断言/证据 | 覆盖边界 |
|---|---|---|---|---|---|---|
| V-01 | AC-01 绑定表可复算 | 冻结提交 + 同工具链 | 表生成脚本两次运行 | 逐字节一致；与注册面指纹一致 | 表 JSON + sha256 | 6 族名字集 |
| V-02 | AC-01 实测覆核 | 最小 Auto 程序（CJK 取 len/byte_len、`Math.sqrt(-1)`、不可用时钟若有构造） | `auto`/VM 实际执行 | 生效实现与表一致；不一致即红 | 程序输出 + 表 | 可构造的名字；不可构造显式 pending |
| V-03 | AC-02 行为零变更 | 同一组程序（CJK 串取 len/byte_len、非法数字解析、`Math.sqrt(-1)`、时钟、HTTP 应答）在处置前后 | VM + a2r/vue 生成调用面 + `plan755` | 除文档/注释外，**用户可观察输出与错误逐字节一致**；任何差异即红（DISTINGUISH 不改行为） | 前后输出字节对照 | 六族全部 |
| V-04 | AC-03 DISTINGUISH 一致性 | 名字/文档/测试 | 规范页 + 代码注释 + 测试 | 三者对同一语义一致；`str.len` 文档必须与实际生效语义相同 | 文本对照 + 定点测试 | DISTINGUISH 族 |
| V-05 | AC-04 契约不破 | error-reply 处置 | `plan755_http_contract_tests` | body 字节不变；全绿 | 测试输出 | HTTP 应答面 |
| V-06 | AC-05 门禁联动 | 冻结三仓组合 | 867 三仓完整档 | 通过；被退役/改名测试在账本内为 RETIRED/RENAMED | 门收据 + 账本条目 | 867 覆盖档 |

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | builtin 绑定表覆盖 6 族全部名字，可复算 + 实测覆核；不可构造项显式 pending |
| AC-02 | work | 六族全部按 DISTINGUISH 落地：每族有证据、名字/文档/测试一致；**行为零变更有对照证据**；MERGE 未实施；RETIRE 若有需单独签核 |
| AC-03 | work | 处置落地后名字/文档/规范/测试一致；`str.len` 三方矛盾消除 |
| AC-04 | work | `plan755` HTTP 契约绿；JS `.length`/生成器调用面按裁定保持 |
| AC-05 | work | 867 三仓完整档复跑通过；测试处置在账本内有据 |
| AC-06 | review | 独立终审绑定冻结 commit/依赖/输入，核 V-01..V-06、SD-01/SD-02、遗漏与健康面；merge 门 pending |
| AC-07 | merge | 冻结输入上跑触面档 + 三仓完整档；SD/ledger 沉淀、归档与清理 |

## 8. 执行步骤

10 项 = 8 work + 1 review + 1 merge；G872=U867∧U869。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | G872 | builtin 绑定表（静态导出 + 最小程序实测覆核）；确认动态注册路径无遗漏 | AC-01 |
| [ ] | T-02 | work | G872；T-01 | 六族 DISTINGUISH 落地材料（消费者/承诺/语义差异/文档口径/是否需补名字）；确认无"完全无消费者的单侧"（否则提 RETIRE 并单独签核） | AC-02 |
| [ ] | T-03 | work | G872；T-02 | 按 DISTINGUISH 实施：补名字/改文档/保持两侧实现，**零行为变更** | AC-02, AC-03 |
| [ ] | T-04 | work | G872；T-03 | 测试：区分性/边界例 + **行为零变更对照** + `plan755` 契约；更新受影响生成器调用面（仅文档/名字，不改行为） | AC-03, AC-04 |
| [ ] | T-05 | work | G872；T-04 | 行为对照收据（两侧程序输出字节）+ 绑定表更新 | AC-01, AC-03 |
| [ ] | T-06 | work | G872；T-05 | 867 三仓完整档复跑 + 账本处置登记 | AC-05 |
| [ ] | T-07 | work | G872；T-06 | SD-01/SD-02 落地 + `attachments/872/handoff.md` 冻结送审 | AC-02, AC-03 |
| [ ] | T-08 | work | G872；T-07 | 全 AC/V 自检与遗漏扫描；未闭项列 finding/pending | AC-01..AC-05 |
| [ ] | T-09 | review | G872；T-08 冻结提交 | 独立复审（绑定表重算、**行为零变更对照重放**、契约测试、文档一致性） | AC-06 |
| [ ] | T-10 | merge | G872；T-09 pass | 冻结输入触面档 + 三仓完整档；SD/ledger、归档、清理 | AC-07 |

**实施树**：`D:/autostack/.wt/lang-872/{auto-lang,auto-ui,auto-man}`（仅需写被裁定的仓），分支 `plan-872-dev`；禁止 junction/symlink；清理前 `wt-guard.sh` clean。
**门禁**：Rust 实现改动（Category B）→ `cargo check -p auto-lang-core` + 定向测试；merge 跑裸 `cargo t` + `tv/tt/tu` 触面档 + 867 三仓完整档。

## 9. 复审记录

2026-10-28 / stage: new / plan_id: PLAN-872 / revision: 1 / outcome: **pass（合同可执行；实施等 U867∧U869 + 用户语义裁定）** / next: 门成立后 work T-01，裁定在 T-02 交用户点选 / changed: 新建 T-01..T-10、AC-01..AC-07、V-01..V-06、SD-01..SD-02。

- 取号：`scripts/new-plan.sh builtin-semantics-dual-track-ruling`（872 → 873）。
- 本轮只写合同：未做实测对照、未定任何族的 MERGE、未建树、未改实现/规范/ledger。
- 已发现并在计划内登记 F-3（`Str.len` 名字/文档/实现三方矛盾），但不预判谁生效。

2026-10-28 / stage: new（r2 语义修订） / plan_id: PLAN-872 / plan_revision: 2 / outcome: **pass（六族处置已裁定为 DISTINGUISH，仍等 U867∧U869）** / ruling: 用户裁定 **六族全部 DISTINGUISH、零行为变更**；MERGE 未授权（D-1 关闭）、兼容口径问题随之不适用（D-2 关闭）/ changed: §1 G2、§2 裁定项、§5.2 默认、§5.3 MERGE 分支、AC-02、T-02/T-03、V-03（改为"行为零变更"对照）、§10 D-1/D-2 / next: 门成立后 work T-01；若未来要合并 `math` 等，另开 revision 并取得裁定。

## 10. 待澄清事项

| ID | 事项 | 影响 | 处置 |
|---|---|---|---|
| D-1 | ~~六族里哪些允许 MERGE~~ **已裁定（2026-10-28）**：全部 DISTINGUISH，MERGE 未授权 | T-03 实施面 | 关闭：按 §5.3 DISTINGUISH 路径实施；未来要合并另开 revision |
| D-2 | ~~MERGE 的兼容口径~~ **本轮不适用**（无 MERGE） | 用户程序兼容 | 关闭；若未来授权 MERGE 再定 |
| D-3 | error-reply 是否值得抽 helper（收益/风险） | 净删与契约风险 | T-03 有界判定：仅当 body 字节不变且净删>0 |
| D-4 | 不可构造的实测项（如不可用时钟）如何记 pending | AC-01 完备性 | 显式 pending + 环境说明，不计 pass |
