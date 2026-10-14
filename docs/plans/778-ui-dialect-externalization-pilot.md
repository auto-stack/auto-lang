---
plan_id: PLAN-778
status: drafting
feature_name: 外部 UI 方言与 ParserBridge 可行性探针
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-13
plan_revision: 1
current_step: 0
total_steps: 7
supersedes_spec_components: [docs/specs/auto-lang/frontend/overview.md]
new_spec_components: []
touched_goals: [GOAL-007]
affects: [docs/specs/auto-lang/frontend/overview.md]
---

# [PLAN-778] 外部 UI 方言与 ParserBridge 可行性探针

## 0. 变更摘要

补Dialect最小外部parser接口、显式注册与错误退出恢复。独立Rust consumer实现真实model文法子集证明可用，默认内建UI兼容路径保留。

## 1. 目标

外部crate不访问pub(crate)字段即可解析 `model { count int = 1 }` 产出typed ModelBlock；session更新/指定嵌套/错误恢复不丢注册。非目标：全UI文法迁移、移除UI AST、改HIR/全部tokens/动态ABI/renderer/新仓。

## 2. 架构方案

继续Dialect返回core Stmt的兼容路线。ParserBridge提供游标/checkpoint/core expr/type/context/diagnostics最小面。外部fixture仅有限model子集；它不是第二份长期UI grammar真值，后续完整迁移时收编。默认UiDialect暂留；显式替换/追加优先规则由T01冻结。

## 3. 技术栈

Rust traits/session/parser/AST，独立fixture manifest显式脱离root workspace或受控exclude。不是动态库Rust ABI插件。

## 4. 需求分析与背景调查

用户在2026-10-13授权调查与报告/规划；本轮不执行重构、不创建新仓、不发布/合入。实施需确认本Plan revision 1。其他在途Plan/WIP保留。

前置777；Specs frontend/ui，Design37；hash基线 reports/v06-modularization-baseline.jsonl。实锚 `dialect.rs::Dialect`、`dialect/ui.rs::UiDialect`、`parser.rs::with_session/build_dialects/try_dialect_stmt/try_token_dialect_stmt/parse_model_block/parse_model_block_inner/parse_model_field_annotations`。pub(crate)状态与mem::take后?提前退出是实需验证点。

## 5. 详细设计

T01冻结一页最小bridge/dispatch/恢复决策，不公开整个Parser。拟新 `src/parser_bridge.rs` 与 `test/fixtures/modularity/ui-dialect-consumer/`；外部实现用bridge解析model字段类型/expr，可选择显式覆盖内建model处理，不能被内建先吃掉导致probe没实际运行。

本pilot支持明确的小子集，注解/widget等走现有实现并列为未迁；不声称其等价。with_session/导入/嵌套创建点逐一记录注册传播范围。Ok(None)不吞token（或已声明且执行rollback），Err恢复registry/context；冲突确定处理。Core view参数模式、task on、普通变量Store和Godot/AutoData必须不变。

### 规范增量

| delta_id | 操作 | docs/specs target | before/after | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/frontend/overview.md | 内建派发→新增实际external注册/bridge支持面与限制，UI AST仍兼容 | 外部证据 | AC-01/02/03 |

## 6. 测试设计

实现者未指派；work自检后交独立上下文review，在同一唯一worktree逐项验收，不修代码；merge仅负责已验收的知识/归档/清理。本草案未派遣agent/chat。

T03创建外部入口 `cargo test --manifest-path test/fixtures/modularity/ui-dialect-consumer/Cargo.toml`；worktree `cargo check -p auto-lang`+裸`cargo t`+`cargo tv`。若改UI codegen追加tu；没触及trans/aavm不要求tt/taa/tf。fixture是拟新，当前未运行。

| V | AC/不变量 | 输入/入口 | oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/external生效 | 外部model + `model { count int = 1 }` | count/int/init=1；不调用core parse_model_block替代自身文法 | 冻结小子集 |
| V-02 | AC-02/Core | `model = 1`、`fn f(view x int)`、task on、既有Godot fixture | 旧行为/诊断保持，不吞变量 | 已支持输入 |
| V-03 | AC-02/持有 | 注册后with_session、嵌套core body、两个Parser实例 | 正确实例注册生效/不串线，禁用场景明确回退 | 明确传播路径 |
| V-04 | AC-02/恢复 | Err/None后同实例到明确恢复点继续合法语句 | registry/context仍在，None不消费token，错误位置正确 | harness建立恢复点 |
| V-05 | AC-03/生产 | 默认UI/Core corpus+冲突反控 | AST/诊断/VM golden无新增漂移，冲突可区分 | 不等完整拆仓 |

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | external真实typed结果；bridge不暴露内部字段；V01 |
| AC-02 | work | None/Err/session/上下文/冲突规则和V02–04实断言 |
| AC-03 | work | 默认兼容/gates无新增红；未迁范围明确；V05 |
| AC-04 | review | 独立AC/API/遗漏审计，bindcommit/revision |
| AC-05 | merge | SD01按pilot真实面沉淀、归档、guard-clean清理 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 操作/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 777+确认r1 | 核读私有能力/grammar；冻结bridge/dispatch/恢复支持矩阵 | 01/02 |
| [ ] | T-02 | work | T01 | parser/dialect/新bridge，显式注册保持与错误恢复 | 01/02 |
| [ ] | T-03 | work | T02 | 独立fixture真实model子集，V01–04正反控 | 01/02 |
| [ ] | T-04 | work | T03 | 默认内建UI保持，V02/V05；点检session/import创建路径 | 02/03 |
| [ ] | T-05 | work | T04 | check+t+tv+externalgate；后续文法迁移清单/限制报告 | 03 |
| [ ] | T-06 | review | T05 | 独立review真实外部面与兼容性 | 04 |
| [ ] | T-07 | merge | reviewpass | Spec/归档/合流/guard-clean清理 | 05 |

拟tree `D:/autostack/.wt/lang-778/auto-lang`，branch `plan-778-dev`；所有实现/构建/测试在该树，主检出只簿记，禁止链接。

## 9. 复审记录

2026-10-13 stage=new，r1，outcome=pass（草案preflight，具体未知由T01有界冻结）；next=确认后work。0/7任务，外部probe/测试未创建运行，不是实现复审pass。

## 10. 待澄清事项

T01只能选择等价接口机制；改默认grammar/扩大model子集/搬UiAst需revision。若必须公开全部lexer/符号表私有面，先出失败决策needs_replan，不以getter全开放绕过。
