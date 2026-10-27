---
plan_id: PLAN-859
status: archived
feature_name: restore-av-corpus-feature-boundaries
author: [Codex]
created_at: 2026-10-27
updated_at: 2026-10-27
plan_revision: 1
completion_kind: superseded
current_step: 0
total_steps: 3
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [auto-test-case]
---

# [PLAN-859] 历史草案：已转入 PLAN-858 Phase 2

2026-10-27 用户要求不用独立 PLAN-859；本草案未实施、未复审、未交付，范围转入 [PLAN-858 r2 Phase 2](../858-autographics-m2-game-samples-and-api-gaps.md)。保留原草案和失败记录供追溯，编号不重用。

## 0. 变更摘要
855 合并后的主检出批量回归在 language listing 编译阶段出现 24 个 E0432/E0433，尚未执行任何测试。保留850全绿覆盖点，以独立修复计划承接，不回滚855。

## 1. 目标
默认/full/book 编译面不消费未启用的 test-cases 依赖；启用 test-cases 的转译档保留821/822/827全部原测试和反例。

## 2. 架构方案
在 crates/auto-lang/src/tests.rs 按模块明确 cfg(feature = "test-cases")；不通过默认启用依赖、删除测试、过滤ID或放宽断言绕过。

## 3. 技术栈
Rust Cargo features；官方 scripts/run-full-regression.py 跨工作区批量入口。

## 4. 需求分析与背景调查
依据 docs/specs/overview.md、docs/specs/auto-test-case/project.md：Case协议由 auto-test-case 提供，auto-lang adapter 消费可选依赖。实际 tests.rs 第35-38行仅822有cfg，821/827裸注册；lib.rs 的 test_cases 和 ac_av_probe_tests 均已正确门控。637037506引入821注册；735f01242在原cfg和821之间插入822，改变属性归属。827也裸注册。精确引入链须在T01补齐，不能将单一插入误报为全部错误起源。
授权：855 merge及批量回归收尾允许登记fix-forward；本计划仅起草，未实施修复。证据 ../plans/attachments/855/batch-r1/receipt.json.txt 与 language-build.log.txt，冻结主线7f966bfa3d416a746b44cfe7abbde897e450e4da。855图形代码仅在autographics-dev，不是该编译失败来源。

## 5. 详细设计
逐个可选依赖消费者挂接cfg，注释对应实际模块，检查同族避免再次属性漂移。

### 规范增量
无公共规范变化：恢复既有可选特性边界，不修改协议/API。若T01发现合同变更，先修订计划。

## 6. 测试设计
work：必要cargo check，以及默认/full选择的编译检查和test-cases启用时三个模块的定向测试。证明默认模块缺席、启用时原测试ID完整，反例仍红转绿。独立review核查依赖消费者和所有验收。
merge：冻结提交后执行适用合入门；最终主检出单实例 cargo tf --batch 完整language/UI/tt/tb，保持原ID、features、profile、limits及报告保护；全绿后才推进批量覆盖锚。开发不运行整档。

## 7. 验收标准
- AC-01 (work/review)：无test-cases默认编译不再出现这24个未解析依赖错误。
- AC-02 (work/review)：test-cases下821/822/827原有ID和实际正反例全部保留并通过。
- AC-03 (merge)：主线官方批量四腿完整执行，回执如实记录；新红归因，不伪造全绿。

## 8. 执行步骤
- [ ] T01 work：定位完整引入链及所有同族消费者，在独立lang-859 worktree修复tests.rs，关闭AC-01/02。
- [ ] T02 review：独立复审实现/证据、无删除或隐式开特性；记录冻结revision。
- [ ] T03 merge：按技能合入、完整主线批量回归、回执与清理；关闭AC-03。

## 9. 复审记录
尚未执行。

## 10. 待澄清事项
无855交付阻塞；本计划待实施。批量回归仍pending，不将未执行测试记作失败或通过。
