---
plan_id: PLAN-1120
status: drafting
feature_name: "AV 泛型 List 高阶方法补齐"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-22
plan_revision: 1
current_step: 0
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

- [ ] AC-01 · owner_stage=work。普通入口非整数 List 的五项操作都完成真实执行，V-01/02 独立判值。
- [ ] AC-02 · owner_stage=work。V-01–03 覆盖所有权、空表、短路和拒绝。旧 ID、源码、旧 oracle 和新证据可逐项追溯。
- [ ] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。
- [ ] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [ ] T-01 核定方法签名与迁移行
  - owner_stage=work。绑定旧 012、1082/1088 正负例和 `std_list_hof`。先用非整数 map/find 复现现状，确认实例化、闭包和资源通路以及旧 find 口形处置。
- [ ] T-02 补齐泛型方法与普通源码用例
  - owner_stage=work。依赖 T-01。在共享绑定与库体实现五项方法，运行 V-01/02，保留实测源序和短路。
- [ ] T-03 验证资源与原整数近邻
  - owner_stage=work。依赖 T-02。执行 V-03，沉淀唯一 canonical 用例与迁移映射。
- [ ] T-04 独立复审
  - owner_stage=review。依赖 T-03。核 AC-03、SD 增量、实测结果和旧用例映射。
- [ ] T-05 冻结验证与归档
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1120 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

## 10. 待澄清事项

T-01 核旧 `find` 直接取字段和列表构造拼写的兼容处置；未经裁定的语义变化保留具体诊断及待决记录。
