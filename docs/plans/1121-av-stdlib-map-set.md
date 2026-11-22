---
plan_id: PLAN-1121
status: drafting
feature_name: "AV 标准库 HashMap 与 HashSet 迁移"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-22
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-972, PLAN-1084, PLAN-1088, PLAN-1096]
supersedes_spec_components: [docs/specs/ace-migration/m2-stdlib-core.md, docs/specs/stdlib/design/ace-collections.md]
new_spec_components: []
touched_goals: [GOAL-001, GOAL-002]
affects: [stdlib, auto-av, auto-semantics]
---

# [PLAN-1121] AV 标准库 HashMap 与 HashSet 迁移

## 0. 变更摘要

2026-11-22 起草。承接 [AV 对 AutoVM 的迁移差额](../design/strategy/av-autovm-language-parity.md)的本领域条目，使用 [PLAN-1096 已交语言基线](archive/1096-av-parity-full-language-certification.md)。

## 1. 目标

在 Windows AV 普通工程交付受检 Map/Set 操作和旧 `HashMap`、`HashSet` 的可用迁移入口。首批覆盖旧用例实际使用的字符串键、字符串/整数值、构造、查询、覆盖/去重、数量、删除和清空。

## 2. 架构方案

公共 Auto 容器接口 → 共享类型实例与所有权检查 → AV 集合存储。先核现有整数 map 和普通泛型记录、拥有值可支撑的底座，复用公共语义。键的相等、散列、缺键和引用规则由公共容器合同给出。

## 3. 技术栈

`stdlib/ace/asm/map.at`、`stdlib/auto/{hashmap,hashset}.at`、`experimental/ace/crates/core/src/source/resolve.rs`、普通集合 AutoDown 语料。

拟实施树为 `D:/autostack/.wt/lang-1121/auto-lang`，分支 `plan-1121-dev`。

## 4. 需求分析与背景调查

[核心标准库](../specs/ace-migration/m2-stdlib-core.md)当前为 `List<int>` 上的整数 map，[1084](archive/1084-av-parity-dynamic-objects-and-null.md)把 typed 容器留给标准库。原 [迁移表](../../crates/auto-lang/test/cases/language/migration/legacy-map.tsv)中 `legacy.13_collections.001–013` 和 `legacy.21_conv.003–004` 为首批；已交付的整数 map、Dyn 对象和其各自行为继续以原用例观察。

取材基线为主检出 `f0d9dcaf3` 及同一检出的活动计划。用户本轮授权在 auto-lang 分类立项，计划状态为草案。

## 5. 详细设计

T-01 用字符串键、局部 owned 字符串值与取值后删除验证现有存储和所有权可行性，确定构造名及 Type/方法回写、键域、alias 规则、缺键返回值和兼容层拼写。原用例中两个删除失败（13.006、13.011）须定位失败原因，保留原记录，按获准后的容器合同分别断言成功删除和缺键。迭代顺序只在规格已承诺时固定。库以纯 Auto 可行路线优先，新增通用容器机制须经 T-01 举证。

### 规范增量

| delta_id | 方式 | 目标 | 规则与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | [`docs/specs/ace-migration/m2-stdlib-core.md`](../specs/ace-migration/m2-stdlib-core.md) | 记录首批公共操作、兼容拼写、已验键值域 | AC-01/02 |
| SD-02 | modify | [`docs/specs/stdlib/design/ace-collections.md`](../specs/stdlib/design/ace-collections.md) | 补充容器资源、键相等、替换/清空和缺键行为 | AC-01/02 |

## 6. 测试设计

下列验收尚未执行。用例由对应 work 任务落到真实公开入口，沿 [共享 AutoDown 语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)保留唯一作者。旧结果作迁移参照，新增断言使用独立预期。

| 用例 | 验收与责任任务 | 输入与路径 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | AC-01 · T-02 | 旧 `legacy.21_conv.003/004` + Map/Set 普通工程 | 输出按原可比预期分别为 `age=30 score=100 has=1 size=1` 与 `apple=1 cherry=0 size=1`，只按显式兼容规则处理展示差异。 |
| V-02 | AC-01/02 · T-02 | 相同字符串键覆盖；不同键发生散列冲突时保留独立键；Set 重复插入 | 覆盖不增加大小，冲突键各得原值，集合重复只计一次，错键值类型在适用的静态入口具名拒绝。 |
| V-03 | AC-02 · T-03 | 删除、清空、缺键、最后 owner 与旧删除红例 | 大小变化、缺键协议、未删另一条、原 owner 存活及最后释放可观察；旧原例按迁移表分类保留。 |

冻结验证命令为 `cargo t ace-stdlib ace-source ace-semantics ace-lowering ace-av case-av-language-parity`。按实际改面核对固定组和下游，分列实跑、原红、排除与未跑。Windows 在 Windows 环境验收，Linux 已有标准库义务进入 [1063](1063-ace-stdlib-linux-deferred-validation.md) 的独立执行台账。

## 7. 验收标准

- [ ] AC-01 · owner_stage=work。在普通 AV 包中运行 V-01 和容器近邻，旧 Map/Set 检查逐行承接。
- [ ] AC-02 · owner_stage=work。V-02/03 的替换、冲突/去重、缺键、生命周期有独立观察，整数 map 和 Dyn 不回归。
- [ ] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。
- [ ] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [ ] T-01 核定 Map/Set 接口与存储可行性
  - owner_stage=work。调查 15 条原用例和现有 ACE 可表达容器。在普通包中跑字符串键和 owned 值探针，先确定接口、容器语义、底座及旧失败归类。
- [ ] T-02 实现容器和普通工程入口
  - owner_stage=work。依赖 T-01。交付公共库、必要的受检接线与旧兼容入口，运行 V-01/02。
- [ ] T-03 补旧用例映射和生命周期证据
  - owner_stage=work。依赖 T-02。运行 V-03、实际整数 map 与 Dyn 近邻，登记各旧行。
- [ ] T-04 独立复审
  - owner_stage=review。依赖 T-03。核 AC-03、SD 增量、实测结果和旧用例映射。
- [ ] T-05 冻结验证与归档
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1121 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

## 10. 待澄清事项

StrBldr 仍沿现有文本和容器台账。T-01 的容器底座决策先于新增共享布局。
