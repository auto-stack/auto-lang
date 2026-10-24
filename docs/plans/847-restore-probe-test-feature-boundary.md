---
plan_id: PLAN-847
status: drafting
feature_name: Restore optional probe-test feature boundary
author: [Codex]
created_at: 2026-10-25
updated_at: 2026-10-25
plan_revision: 1
current_step: 0
total_steps: 4
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# [PLAN-847] 恢复 probe 测试的可选依赖边界

## 0. 变更摘要

845 落地后的批量回归发现主分支 feature-off 编译红，修复独立于已交付的图形源码。此计划仅起草，尚未实施。

## 1. 目标

恢复 `probe_adapter_tests` 对 `test-cases` 可选依赖的原有门控；feature-on 保留旧 probe 与新增 MIR-native 两族完整测试选择面。不得通过默认启用依赖、删除测试、ignore 或放宽时限变绿。

## 2. 架构方案

维持 auto-lang → optional auto-test-case 依赖方向；各消费可选依赖的测试模块分别绑定 feature。仅预期修改 `crates/auto-lang/src/tests.rs` 及必要的可维护回归断言，不改生产协议、阈值或测试档选择。

## 3. 技术栈

Rust 条件编译、Cargo features、nextest；Windows 主检出批量回归。

## 4. 需求分析与背景调查

- 授权来源：用户要求 845 完整合入与归档；auto-plan-merge 触发 auto-plan-regress，后者明确要求新红建立 fix-forward Plan。本草案不把此授权扩大为实施 847。
- 背景：`docs/specs/overview.md`、`docs/specs/auto-test-case/project.md` 与 `architecture.md` 规定协议核心和 adapter 依赖边界；`docs/specs/auto-lang/project.md` 规定 feature-on/off 编译与完整 ID 面分别核验。
- 观察基线：845 批量冻结主提交 `c9ab21ea69ff70ed8f53bbbef87f9a89ff26b891`；上次完整绿 `53298cd273e0c04fabc508745ce342655394f1a4`。原始输出见 [tf](attachments/845/batch-r1/tf.txt)，三档终态见 [receipt](attachments/845/batch-r1/receipt.json.txt)。
- 引入提交：`5bebb0ad02c9807f157968bc6b34bf59f7431d66`（818 T-02）在旧 `#[cfg(feature = "test-cases")]` 与 `mod probe_adapter_tests;` 之间插入新模块，两个 cfg 同附新模块，旧模块失去门控。Cargo.toml 的 auto-test-case 为 optional，default/tb 不启用，tt 启用。
- 845 图形源码仅落 autographics-dev，未修改这些根 Rust 文件。818 本身仍有独立验收/落地流程；不以本草案豁免其门禁。
- 工作树待实施时创建：`D:/autostack/.wt/lang-847/auto-lang`，`plan-847-dev`；当前不创建。

## 5. 详细设计

T-01 先核对实际基线及同族模块所有 optional 引用；预期为局部门控修复。若发现生产依赖或其他错误，先记录范围，不搭便车修复。保留新增 ac_mn 与既有 probe 的 feature-on 执行路径。

### 规范增量

| delta_id | 操作 | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | 无变更 | docs/specs/auto-test-case/architecture.md; docs/specs/auto-lang/project.md | 仍按既有 feature-on/off 与可选依赖规则执行 | 恢复已约定边界，不增加产品支持或修改规范；review 核实 | AC-01..03 |

## 6. 测试设计

区分错误实现：只给新模块叠 cfg 无法修复旧模块；默认开启依赖掩盖 feature-off；屏蔽整族会丢失 feature-on ID。work 运行 feature-off `cargo check -p auto-lang --tests` 及 feature-on 定向编译/既有模块回归，记录完整测试 ID 对比；不得在开发或 review 运行整档 t/tf/tt/tb。review 在独立上下文核对代码、断言与 ID；不指定模型。merge 在冻结提交执行裸 cargo t 和必要触面档，并按批量技能在主检出单实例重跑未完成 tf/tt/tb。原输入、P95 和窗口断言均不改。

## 7. 验收标准

| ID | owner_stage | 可观察标准与验证 |
|---|---|---|
| AC-01 | work | feature-off --tests 编译成功，auto-test-case 仍为 optional/default-off；恢复旧失门控即重新报 unresolved import |
| AC-02 | work | feature-on 两族既有 ID 全保留、相关定向测试通过；不改时限、输入、oracle 或配置选择面 |
| AC-03 | review | 独立上下文按实际代码/依赖/测试结果与 Spec 核实，无未批准延期；明确 merge 门 pending |
| AC-04 | merge | 冻结整档与未完成批量门实际执行，记录每档真实通过或归因结果，规范/账本/归档/guard 按技能完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 位置与结果 | 验收 |
|---|---|---|---|---|
| T-01 | work | 用户启动实施 | 现有 tests.rs/Cargo.toml 与完整模块依赖扫描，冻结修复范围 | AC-01/02 |
| T-02 | work | T-01 | 专用工作树修门控、补反控/选择面证据、定向验收并提交 | AC-01/02 |
| T-03 | review | T-02 | 同一树独立上下文审查，冻结 revision/commit 与不变 Spec 结论 | AC-03 |
| T-04 | merge | T-03 pass | 合入整档、主检出批量恢复、知识/收据/归档/guard 清理 | AC-04 |

四步均未执行，当前 0/4。

## 9. 复审记录

草案自查：生产范围与 845 图形交付分离，四项验收均有负责阶段与观察点；未运行 847 实施/测试/独立审查，不把提议当证据。

## 10. 待澄清事项

无阻碍起草的问题。实施需由用户启动 847；批量原始证据和新红债务保留，旧批量完整绿覆盖点不得冒进到编译失败提交。
