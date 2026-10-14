---
plan_id: PLAN-777
status: drafting
feature_name: v0.6 模块边界基线与增量检查
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-13
plan_revision: 1
current_step: 0
total_steps: 6
supersedes_spec_components: [docs/specs/design/repository-layout.md]
new_spec_components: []
touched_goals: []
affects: [docs/specs/design/repository-layout.md]
---

# [PLAN-777] v0.6 模块边界基线与增量检查

## 0. 变更摘要

建立可复现的规模、依赖与资产基线，以及新增坏依赖检查。仅脚本/配置/文档，不改Rust、Cargo构建或语言行为。

## 1. 目标

调查可重复，现有焊点有owner/退出阶段，新增跨界依赖可确定性判红。非目标：搬代码/建仓/换AST/VM/性能优化或统计全仓精确重复率。

## 2. 架构方案

区分normal/build/dev/optional声明边、源码文本候选和人工确认；观察模式覆盖全仓，strict仅判冻结的新增边。存量窄baseline不豁免整个vm/lib目录。依据Design37，目标态不得当成当前已经达成的事实。

## 3. 技术栈

Python标准库、Git tracked files、Cargo metadata（不编译）。

默认以 `cargo metadata --no-deps --locked --format-version 1` 读取；如锁状态不满足，记录依赖调查失败/缺口，不自动更新。实际 feature 链抽查同样用 `cargo tree --locked`，且不把该命令当编译验收。

## 4. 需求分析与背景调查

用户在2026-10-13授权调查与报告/规划；本轮不执行重构、不创建新仓、不发布/合入。实施需确认本Plan revision 1。其他在途Plan/WIP保留。

前置报告 reports/v06-modularization-research.md、逐文件hash reports/v06-modularization-baseline.jsonl、Design37、Specs overview/README与repository-layout。调研HEAD 5e21561d2c30，src物理行704000；work重采HEAD，不固化该行数为永久阈值。

## 5. 详细设计

拟新 `scripts/modularity-audit.py`、`.autoos/module-boundaries.toml`、`test/fixtures/modularity/audit/`。输出schema/method/revision/file hashes、声明边/文本候选/unknown。稳定payload不含timestamp/绝对检出路径；baseline绑定base revision，过期或宽豁免不能默默通行。输入只读，生成报告另存。

ownership表覆盖报告的六类UI焊点、服务/runtime非UI归属、资产、外部consumer。测试/生成无法精确分类时写unknown，不报虚假SLOC。新增边与存量边分开，允许合法UI→SDK与dev-only，不把文本match当语义解析。

### 规范增量

| delta_id | 操作 | docs/specs target | before/after | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/design/repository-layout.md | 增加真实审计入口/口径/baseline规则，不把目标仓写成已拆 | 可复现治理 | AC-01/02/03 |

## 6. 测试设计

实现者未指派；work自检后交独立上下文review，在同一唯一worktree逐项验收，不修代码；merge仅负责已验收的知识/归档/清理。本草案未派遣agent/chat。

Category A：禁止cargo t/docs_gen。T01实现后 `python scripts/modularity-audit.py --help`；T03实现拟新 `--self-test`；T04运行真实扫描与绑定revision的差分。metadata成功不代表编译成功。

| V | AC/不变量 | 输入/入口 | 可观察oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/稳定只读 | 同HEAD扫描两次，前后hash/status | payload相同且输入不变 | tracked物理行 |
| V-02 | AC-02/新增边 | fixture实际新增core→UI正常边 | 非零+具体path/edge；合法反向/dev-only不误红 | manifest声明边 |
| V-03 | AC-03/候选诚实 | 注释/测试/alias/宏/string对照 | 分类与盲区明确，不声称完整Rust解析 | 文本scanner |
| V-04 | AC-02/baseline | 窄豁免/撤销边/过期revision/全目录豁免 | 存量不能盖新边；过期/宽豁免失败 | 先证明fixture被改 |

## 7. 验收标准

| ID | owner_stage | 可观察结果与验证 |
|---|---|---|
| AC-01 | work | 可复现真实报告schema/method/hash/revision；V01 |
| AC-02 | work | 窄baseline+退出owner；V02/04实际判红 |
| AC-03 | work | ownership覆盖全部调研焊点/资产/consumer；V03未知范围明确 |
| AC-04 | review | 独立逐AC审计/遗漏检查，绑定commit与revision |
| AC-05 | merge | 真实SD01沉淀、归档/账本更新、wt-guard clean后清理 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 路径/操作与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 确认r1 | 新audit脚本，重采HEAD/文件，help+V01 | 01 |
| [ ] | T-02 | work | T01 | 新boundary配置/ownership，逐项核对真实路径和存量退出阶段 | 02/03 |
| [ ] | T-03 | work | T02 | 新audit fixtures/self-test，V02–04实际正反控 | 02/03 |
| [ ] | T-04 | work | T03 | 真实扫描/diff/check输入不变，受管evidence；不跑Rust tests | 01/02/03 |
| [ ] | T-05 | review | T04 | 独立review，禁止baseline偷换目标 | 04 |
| [ ] | T-06 | merge | review pass | Spec/归档/合流/受保护清理；batch到期按现行规则 | 05 |

拟tree `D:/autostack/.wt/lang-777/auto-lang`，branch `plan-777-dev`；脚本实现也必须在worktree。Plan簿记在master；依赖用new-wt-group，禁止链接。

## 9. 复审记录

2026-10-13：stage=new；r1；outcome=pass（草案preflight）；next=确认实施后work。0/6 task，脚本/fixture/测试均未创建执行；不是独立实现复审pass。

## 10. 待澄清事项

仓名/发布渠道不影响baseline。任何strict扫描扩面要先有可区分错误实现的oracle，再修订contract；不把此Plan扩大成Rust编译器。
