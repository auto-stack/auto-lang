---
plan_id: PLAN-1120
status: reviewed
feature_name: "AV 泛型 List 高阶方法补齐"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-23
plan_revision: 1
current_step: 3
total_steps: 5
depends_on: [PLAN-972, PLAN-1082, PLAN-1088, PLAN-1096]
supersedes_spec_components: [docs/specs/auto-av/language-parity.md, docs/specs/ace-migration/m2-stdlib-core.md]
new_spec_components: []
touched_goals: [GOAL-001, GOAL-002]
affects: [auto-av, auto-semantics, stdlib]
---

# [PLAN-1120] AV 泛型 List 高阶方法补齐

## 0. 变更摘要

2026-11-22 起草。承接 [AV 对 AutoVM 的迁移差额](../design/strategy/av-autovm-language-parity.md)的本领域条目，使用 [PLAN-1096 已交语言基线](archive/1096-av-parity-full-language-certification.md)。

## 1. 目标

在 Windows 普通 AV 源码和包入口上，扩展 `List<T>.map/reduce/find/any/all` 到文本与记录等已准入的元素类型。`map` 可由 `T` 产生不同的 `U`，`reduce` 的累加类型可与元素类型不同。保留 `filter/to_array` 及 `List<int>` 的既有行为。

## 2. 架构方案

普通 Auto 源 → 共享绑定和泛型实例 → 受检 HIR/MIR → AV。优先复用 1088 的元素/资源准入、1082 的列表方法解析及 972 的标准库算法。实现可复用同一组泛型体或既有受检机制，公共方法行为和所有权契约只定义一份。

## 3. 技术栈

`experimental/ace/crates/core/src/source/resolve.rs`（`std_list_hof`、`filter_typed_list`）、`stdlib/ace/asm/list.at`、公共 `language/` AutoDown 语料。

拟实施树为 `D:/autostack/.wt/lang-1120/auto-lang`，分支 `plan-1120-dev`。

## 4. 需求分析与背景调查

[972 核心标准库](../specs/ace-migration/m2-stdlib-core.md)与[1082/1088 语言规范](../specs/auto-av/language-parity.md)记录既有合同。主检出 `std_list_hof` 对非 `VT::I32` 报 `source.hof-elem-domain`，`filter` 有单独的泛型分支。旧 [`legacy.25_method_u64.012_list_hof_value`](../../crates/auto-lang/test/vm/25_method_u64/012_list_hof_value/list_hof_value.case.ad)覆盖 `List<Note>` 整组方法。该行归本计划，宽整数方法族的 001–011 归 [1122](1122-av-stdlib-wide-unsigned-conversion.md)。

取材基线为主检出 `f0d9dcaf3` 及同一检出的活动计划。用户本轮授权在 auto-lang 分类立项，计划状态为草案。

## 5. 详细设计

冻结闭包参数、返回值、源序、空表、`find` 未命中和所有权。沿既有 AV `find` 的 Option 合同；原例直接取字段的旧写法须逐项核对，若需要迁移，保留原源和精确诊断，另加显式 Option 处理的结果用例。保留 `map/filter` 的元素访问顺序及 `any/all/find` 短路。捕获资源、结果和原表的所有权经正常结束与谓词报错观察。比较闭包排序和负索引保留原归属。

### 规范增量

| delta_id | 方式 | 目标 | 规则与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | [`docs/specs/auto-av/language-parity.md`](../specs/auto-av/language-parity.md) | 给出非整数元素的 HOF 支持域、回调与持有规则 | AC-01/02 |
| SD-02 | modify | [`docs/specs/ace-migration/m2-stdlib-core.md`](../specs/ace-migration/m2-stdlib-core.md) | 将整数实例限制更新到实测公共方法覆盖 | AC-01/02 |

## 6. 测试设计

下列验收尚未执行。用例由对应 work 任务落到真实公开入口，沿 [共享 AutoDown 语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)保留唯一作者。旧结果作迁移参照，新增断言使用独立预期。

| 用例 | 验收与责任任务 | 输入与路径 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | AC-01/02 · T-02 | 旧 `List<Note>`，以及 `map(Note→int)` / `reduce(int, Note)` | 固定独立结果 `[2,2,world,true,true,3,1,3,6]`，按各端合同核布尔展示。原 VM oracle 保留，AV 显式兼容改写须绑定原行和原源诊断。 |
| V-02 | AC-01/02 · T-02 | 空 `List<Str>`、未命中、短路计数和回调类型错误 | 零回调、reduce 保持初值、find=None、any=false、all=true。短路后的回调不执行，类型错在绑定时具名拒绝。 |
| V-03 | AC-02 · T-03 | 带拥有型字段的记录及可捕获回调错误 | 两个集合的资源存活按合同独立，失败后临时结果释放，输入可按其所有权继续使用。原 int 用例继续通过。 |

冻结验证命令为 `cargo t ace-source ace-semantics ace-lowering ace-av ace-stdlib case-av-language-parity`。按实际改面核对固定组和下游，分列实跑、原红、排除与未跑。Windows 在 Windows 环境验收，Linux 已有标准库义务进入 [1063](1063-ace-stdlib-linux-deferred-validation.md) 的独立执行台账。

## 7. 验收标准

- [x] AC-01 · owner_stage=work。普通入口非整数 List 的五项操作都完成真实执行，V-01/02 独立判值。
- [x] AC-02 · owner_stage=work。V-01–03 覆盖所有权、空表、短路和拒绝。旧 ID、源码、旧 oracle 和新证据可逐项追溯。
- [x] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。（2026-11-23 复审 r2 通过，见 §9）
- [ ] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。
- [ ] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [x] T-01 核定方法签名与迁移行
  - owner_stage=work。绑定旧 012、1082/1088 正负例和 `std_list_hof`。先用非整数 map/find 复现现状，确认实例化、闭包和资源通路以及旧 find 口形处置。
- [x] T-02 补齐泛型方法与普通源码用例（2026-11-23 复审 F1/F2 修复后关闭）
  - owner_stage=work。依赖 T-01。在共享绑定与库体实现五项方法，运行 V-01/02，保留实测源序和短路。
- [x] T-03 验证资源与原整数近邻（2026-11-23 复审 F1 文本域资源断言补齐后关闭）
  - owner_stage=work。依赖 T-02。执行 V-03，沉淀唯一 canonical 用例与迁移映射。
- [ ] T-04 独立复审
  - owner_stage=review。依赖 T-03。核 AC-03、SD 增量、实测结果和旧用例映射。
- [ ] T-05 冻结验证与归档
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1120 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

2026-11-23 · stage: work | PLAN-1120 r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1120/auto-lang | branch: plan-1120-dev | base_commit: d4bc60a89 | code_commit: d47bd50dc | dependency_revisions: 无跨仓改动 | task_ids: T-01/T-02/T-03 | blockers: 无 | next_responsible_role: review。

work 证据（原始日志：D:/autostack/plan-artifacts/plan-1120/，合并收尾时随目录删除）：

- **T-01 基线复现**：主检出 `std_list_hof` 对非 I32 元素报 `bind/source.hof-elem-domain`（resolve.rs:13508 旧行；plan1082 `hof_non_int_elements_reject` 负控钉住）。旧 `find` 直接取字段（`found.id`）在 Option 合同下为 `bind/source.type-mismatch`，V-01 迁移为显式 `is` 处理（保留原源与诊断）。
- **红相**（red-phase3.log）：新增 8 例全部以 `source.hof-elem-domain` 失败，int 近邻 1 例通过——旧合同钉住后才动实现。
- **实现**：`resolve.rs` 新增 `hof_typed_callback`（共享回调准入）与 `map_typed_list`/`find_typed_list`/`any_all_typed_list`/`reduce_typed_list` 四个内联脱糖（snapshot 克隆隔离、源序访问、`find` 绑 `?T` Option、`any/all` 短路带具名 `Break{target: Some(..)}`、`reduce` 累加类型取自 init）；`resolve_list_method` 在非 I32 + auto_language 面分派到新车道，`List<int>` 冻结 graft 不动。修复过程中发现并规避两个陷阱：脱糖 break 必须带具名 LoopRef（`target: None` 走 phantom-loop 具名拒车道，adapter.rs phantom_target）；`all` 的停止条件为谓词取反。
- **绿相**（green-phase7/8）：plan1120 族 9/9 通过（V-01 独立预期 stdout `2 2 world true true 3 1 3 6`；V-02 空表零回调/未命中/短路计数 p×1 q×1 r×2 与绑定拒绝；V-03 全所有者释放、fail-Nth 分配故障重放、回调 `fail` 经 try/catch 后输入续用返回 61；int 近邻 42）。
- **近邻族**（scoped-2/3.log）：plan1082 17/17（含迁移后的 `hof_non_int_callback_type_rejects`）、plan1088 3/3、plan1086 10/10、plan1083 3/3、plan1081 21/21。`cargo check -p ace-core` 零错误、无新增警告。
- **迁移与规范**：legacy-map.tsv `012_list_hof_value` 行归本计划（canonical `av.parity.1120.generic-list-hof`）；SD-01/SD-02 增量已写入 `docs/specs/auto-av/language-parity.md`（PLAN-1120 增量节）与 `docs/specs/ace-migration/m2-stdlib-core.md`（PLAN-1120 增量节 + 1082 边界行修订）。
- **待办（merge 所有）**：§6 冻结固定组并集（`cargo t ace-source ace-semantics ace-lowering ace-av ace-stdlib case-av-language-parity`）在冻结提交上执行；本阶段未跑全量，按纪律记为 pending。

2026-11-23 · stage: goal | plan_id: PLAN-1120 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1120/auto-lang | commit: d47bd50dc | next: review

2026-11-23 · stage: review | review_scope: final（本计划无分立终审角色；实现会话内自复审，局限已记——结论以制品重放重建，未采信执行者摘要） | reviewer: goal-session main agent（同 work 会话，独立性受限） | PLAN-1120 r1 | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1120/auto-lang | branch: plan-1120-dev | reviewed_commit: d47bd50dc | base_commit: d4bc60a89 | dependency_revisions: 无 | spec_inputs: docs/specs/auto-av/language-parity.md（worktree 副本，PLAN-1120 增量节）、docs/specs/ace-migration/m2-stdlib-core.md（同） | acceptance_results: AC-01 partial（记录域五项真实执行+V-01 独立判值 PASS；文本域仅空表路径，非空 StrOwned 循环体零执行）/AC-02 partial（所有权/短路/拒绝/迁移在记录域闭合；文本域资源断言缺失；谓词返回类型拒绝退到 verify 阶段）/AC-03..04 review/merge 所有未评 | pending_closeout_gates: §6 冻结固定组并集（merge 执行） | findings: F1 major（AC-01/02，T-02/T-03）/F2 minor（V-02，T-02） | evidence: 见下 | next: work（修复范围限 F1/F2 受影响项；近邻族证据复用）

复审证据（本会话重放，基线 d47bd50dc 工作树干净、无未提交实现）：

- 行为重放：nextest `test(plan1120_generic_list_hof)` 9/9 PASS（run c20be340），逐例计时 0.081s–0.175s（gate ≤10s 全过；最慢 hof_short_circuit 0.175s）。测试目录盘点：唯一触及 List<Str> 的用例是 `hof_empty_list_zero_callbacks_and_reduce_keeps_init`，且表为空——脱糖循环体（StrOwned 每访问克隆 / `fn(Str)` 借用视图 / `?Str` Option 槽）零执行、无文本元素 assert_released。
- 代码审查：`hof_typed_callback` 只校验回调参数形（单参 + 元素类型/Str 借用视图），不校验返回类型；`map` 的 U 与 `reduce` 的 acc 分别有取结果/校验，但 `find/any/all` 的谓词结果类型未在校准入校验。非 Bool 谓词当前的拒绝来自 typecheck.rs:190 While 条件检查（Stage::Verify 的 `source.type-mismatch`，落在生成代码上），而非 V-02 约定的绑定时具名拒；filter_typed_list 的先例在准入即查 `result == VT::Bool`。
- 其余审查面干净：分派门（非 I32 + auto_language）不触冻结面；`List<int>` graft 与字节稳定单测未动；`pop/to_array/filter` 原行为保留；调试构建、无隐式日构建路径；规范增量内容与实现一致、frontmatter 影响字段已填；legacy-map 012 行迁移正确。时机表/会话表：准备 0s（复用已构建产物），发现 0.19s，执行 0.179s，无清理；占用观察：运行时另有他计划 cargo/rustc 进程（PID 84804/92584/40248，03:24:57 起），本重放为缓存快run、无资源争用。
- 发现 F1（major，AC-01/AC-02，T-02/T-03）：目标域含「文本与记录」（§1），文本元素的非空执行路径无任何真实执行证据。纠正：新增一例永久用例（建议名 `hof_text_family_real_execution`），非空 `List<Str>`/`List<StrOwned>` 上跑 map/reduce/find/any/all 五项，`assert_released` 钉文本所有者账目，独立预期值（如 `["a","bb","ccc"].map((s Str) => s.len()) = [1,2,3]`、`reduce(0,...) = 6`、`find( len==2 )` 命中、`any/all` 短路计数）。
- 发现 F2（minor，V-02/AC-02，T-02）：谓词返回类型非 Bool 时拒绝退至 verify 阶段生成代码（typecheck.rs While 条件检查），不满足「绑定时具名拒」与 filter 先例。纠正：`hof_typed_callback` 为 find/any/all 准入加 `signature.result == VT::Bool`（map 保持任意 U；reduce 维持现有 acc 校验），具名拒沿用 `source.type-mismatch`。
- 重审范围：仅重放 F1/F2 受影响用例与新用例 + 复用未变近邻证据（plan1082/1086/1088 族）；§6 冻结并集仍归 merge，不随本轮重复。

2026-11-23 · stage: goal | plan_id: PLAN-1120 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1120/auto-lang | commit: d47bd50dc | next: work

2026-11-23 · stage: work（修复轮，F1/F2） | PLAN-1120 r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1120/auto-lang | branch: plan-1120-dev | base_commit: d4bc60a89 | code_commit: 7a114b480 | task_ids: T-02/T-03 | blockers: 无 | next_responsible_role: review（仅重审 F1/F2 受影响项）。

修复证据：

- F2 闭合：`hof_typed_callback` 对 find/any/all 准入增 `signature.result == VT::Bool`（map 保任意 U、reduce 维持 acc 校验），非 Bool 谓词在绑定时具名拒 `source.type-mismatch`。负控：`hof_callback_type_mismatch_rejects_at_bind` 新增 find/all 非 Bool 谓词两例（stage=bind）。
- F1 闭合：新增 `hof_text_family_real_execution`（非空 `["a","bb","ccc"]` 上 map/reduce/find/any/all 五项真实执行 + `assert_released` 文本所有者账目，独立预期 33623）与 `hof_text_family_short_circuits`（p×1/q×1/r×2 短路计数）。
- 排障记录（环境归因，非代码缺陷）：修复轮中一度观测到 all 谓词 q×2 的"伪失败"，隔离探针（expr/block/print/for-in 四形）全部正确后定位为本机时钟偏斜导致 cargo 用过期 ace-core rlib 链接（失败输出恰为修复前 Not 极性行为）；矫正 mtime 强制重建后消失。教训：本机 cargo 增量可靠性受时钟影响，每轮验证须确认重建发生（观察构建时长）并以行为探针复核可疑失败。
- 修复轮验证：plan1120 族 11/11（0.14s）；近邻 plan1082 17/17、plan1088 3/3、plan1086 10/10、plan1083 3/3、plan1081 21/21 复用性重跑全绿。新增 2 例计时 0.1s 级（≤10s 门）。冻结并集仍归 merge。

2026-11-23 · stage: goal | plan_id: PLAN-1120 | plan_revision: r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1120/auto-lang | commit: 7a114b480 | next: review

2026-11-23 · stage: review | review_scope: final（复审 r2，限 F1/F2 受影响项；同会话自复审局限已记） | PLAN-1120 r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1120/auto-lang | branch: plan-1120-dev | reviewed_commit: 7a114b480 | base_commit: d4bc60a89 | dependency_revisions: 无 | spec_inputs: 同 r1（增量内容未随修复轮变动，哈希基线 d47bd50dc 文件副本） | acceptance_results: AC-01 pass / AC-02 pass / AC-03 pass / AC-04 merge 所有 | pending_closeout_gates: §6 冻结固定组并集（merge 在冻结提交执行） | findings: F1/F2 均闭合（closure rows 如下） | evidence: 见下 | next: merge

复审 r2 证据（基线 7a114b480，工作树干净；重建已确认——编译 10.51s 非缓存）：

- F2 closure row：谓词回调准入不变量 = find/any/all 仅收 Bool 返回、绑定时具名拒。修复 diff 核验（hof_typed_callback 增 `(!bool_result || signature.result == VT::Bool)`，消息区分谓词/非谓词）；永久负控 4 子例重放 PASS（map 参型不符、reduce 参型不符、find 非 Bool、all 非 Bool，stage 均 = bind）；合法兄弟路径不受影响重放 PASS（map 任意 U：hof_text_family_real_execution 的 map(→int)；记录域谓词：V-01 族）。
- F1 closure row：文本域非空表真实执行 + 所有者账目不变量。`hof_text_family_real_execution`（33623 独立预期 + assert_released ≥3）与 `hof_text_family_short_circuits`（p×1/q×1/r×2）重放 PASS；隔离探针四形态（expr/block/print/for-in）在修复轮已全对。
- 修复回归面：hof_typed_callback 为五操作共享，受影响域 = plan1120 全族，已整族重放 11/11 PASS（run ca2abe25，逐例 0.099s–0.187s，新增 2 例 0.138s/0.162s ≤10s）；近邻五族 54 例证据自修复轮复用（diff 仅限该 helper 与测试文件，输入/配置未变）。
- 会话计时表：准备 10.51s（编译，已确认非缓存）、发现含于编译、执行 0.192s、无清理；无资源等待（占用观察：无本计划外活动构建冲突）。

2026-11-23 · stage: goal | plan_id: PLAN-1120 | plan_revision: r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1120/auto-lang | commit: 7a114b480 | next: merge

## 10. 待澄清事项

（2026-11-23 T-01 已决，无待澄清）旧 `find` 直接取字段的处置：AV `find` 绑定 `?T` Option 合同，`found.id` 直取 = `bind/source.type-mismatch` 具名拒；迁移行保留原源与诊断，canonical 用例改用显式 `is` 处理（见 §9 work 证据与 legacy-map.tsv 012 行）。
