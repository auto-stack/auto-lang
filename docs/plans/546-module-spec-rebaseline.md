---
plan_id: PLAN-546
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: Auto-Lang Module Spec Rebaseline
author: [Codex]
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects:
  - auto-lang/frontend
  - auto-lang/types
  - auto-lang/comptime
  - auto-lang/interpreter
  - auto-lang/vm
  - auto-lang/trans
  - auto-lang/runtime
  - auto-lang/ui
  - auto-lang/mcp
current_step: 1
total_steps: 23
---

# [PLAN-546] Auto-Lang Module Spec Rebaseline

## 变更摘要

在 PLAN-543 建立仓库级知识库基线后，对 `auto-lang` 的九个 module spec 做一次
代码证据驱动的重新基线：逐模块确认真实边界、主入口、当前/实验/计划状态、测试证据与
已知限制；修正 `docs/specs/auto-lang/`、与其直接对应的 `docs/design/` 文档以及全局入口，
并形成可复现的模块同步报告。本计划只修改知识文档和生成索引，不改变产品实现。

## 目标

1. 九个 module spec 的 overview/architecture/design 与当前 `master` 实现一致。
2. 清除 PLAN-543 已确认的典型漂移：旧 TreeWalker/Evaluator 叙事、VM 32-bit 栈与
   “约 120 opcode”等未经代码支持的当前事实。
3. 为每个重要结论记录代码或测试证据，区分 `current`、`experimental`、`planned`、
   `historical`，不把计划目标写成已实现能力。
4. 让 `docs/specs/overview.md`、`docs/specs/auto-lang/project.md`、九模块 spec 与直接
   对应的 design 文档形成一致的入口链。
5. 产出后续 lint/catalog 自动化可消费的漂移样本与规则候选，但不在本计划实现自动化。

## 架构方案

采用“代码证据 → 模块事实表 → canonical spec → 解释性 design → 全局入口”的单向校准：

```text
Rust/Auto 源码 + 测试 + Cargo workspace
                │ 取证
                ▼
docs/reports/module-spec-rebaseline-2026-09-04.md
                │ 逐项裁定 current / experimental / planned / historical
                ▼
docs/specs/auto-lang/<module>/{overview,architecture,design/*}.md
                │ 摘要与链接
                ▼
docs/design/* + docs/specs/auto-lang/project.md + docs/specs/overview.md
```

校准规则：

- 当前能力必须有生产代码入口和/或可运行测试证据；仅有旧 plan/design 不构成证据。
- 未来方向保留在 design 或 plans 中，并显式标为 `planned`，不能混入当前能力清单。
- 不手工维护易漂移的文件数、opcode 数等计数；必须保留数字时附复现命令和快照日期。
- `plans.md` 只保留模块路线与归档链接，不复制活跃 plan 状态；活跃状态以 frontmatter 为准。
- 发现实现缺陷只登记到 `docs/plans/KNOWN-DEBT-AND-RISKS.md`，本计划不顺手改代码。

## 需求分析与背景调查

`docs/specs/overview.md`（2026-09-04、PLAN-543）把 `crates/auto-lang` 划分为九个模块：
frontend、types、comptime、interpreter、vm、trans、runtime、ui、mcp。当前总览已完成仓库级
入口校准，但明确把模块细节留给后续 rebaseline。

当前并行开发线中，PLAN-532 正在推进 AAVM/self-host，PLAN-536 正在修改 VM/AutoUI
reactive runtime。二者会影响 compiler/VM/UI/runtime 的事实基线，因此本计划的硬前置是：
两计划都已完成合并，或由用户明确裁定某一未合并分支不进入本次基线。取证必须基于该
裁定之后的 `master`，不得从在途 worktree 抄写“将要实现”的状态。

模块与主要代码证据范围：

| 模块 | module spec | 主要实现证据 |
|---|---|---|
| frontend | `docs/specs/auto-lang/frontend/` | `lexer.rs`、`parser.rs`、`ast/`、`resolver.rs`、`macro_/`、`dialect/` |
| types | `docs/specs/auto-lang/types/` | `infer*`、`type*`、`ownership/`、`trait_checker*`、`symbols.rs` |
| comptime | `docs/specs/auto-lang/comptime/` | `comptime/`、`ast/comptime.rs`、`compile.rs` |
| interpreter | `docs/specs/auto-lang/interpreter/` | `execution_engine.rs`、`vm.rs`、模板执行入口及相关测试 |
| vm | `docs/specs/auto-lang/vm/` | `vm/`、`autovm*.rs`、`bigvm*`、VM 测试 |
| trans | `docs/specs/auto-lang/trans/` | `trans/`、各目标 codegen、转译 fixture/金样测试 |
| runtime | `docs/specs/auto-lang/runtime/` | `runtime.rs`、`scope*.rs`、`session.rs`、`host.rs`、`ffi.rs`、`database/` |
| ui | `docs/specs/auto-lang/ui/` | `aura/`、`ui/`、`ui_gen/`、`a2ui/`、桌面运行时测试 |
| mcp | `docs/specs/auto-lang/mcp/` | `mcp/`、MCP server 入口与协议/会话测试 |

本计划不包含：生成 catalog/CI lint、修改 auto-plan 四技能、重写 `docs/design/raw/` 历史
素材、修复审计中发现的产品代码问题。这三类分别归后续自动化、Auto-plan v3 和债务计划。

## 详细设计

### 1. 证据报告

新增 `docs/reports/module-spec-rebaseline-2026-09-04.md`，每模块统一记录：边界、生产入口、
公共 API/关键数据流、当前能力、实验能力、未实现/计划项、测试证据、与旧文档的差异、
本次修改文件。报告中的命令必须能在仓库根复现。

### 2. Module spec 修订

逐模块审查 `overview.md`、`architecture.md`、`design/*.md` 和 `plans.md`：只改有证据的
漂移；保留正确内容；删除或降级没有实现证据的断言；把历史实现明确放入历史段。对尚无
`architecture.md` 的模块，只有在存在跨文件稳定结构且 overview 无法清晰表达时才新增。

### 3. Design 与入口回写

根据模块裁定检查 `docs/design/01-architecture.md`、`02-type-system.md`、
`03-error-handling.md`、`04-memory-ownership.md`、`05-vm-runtime.md`、
`06-code-generation.md`、`08-ui-systems.md`、`09-compiler.md`、`10-language-syntax.md`、
`12-concurrency.md`、`13-networking.md` 及 `docs/design/autoui/` 中被模块事实直接推翻的
当前态描述。未被代码证据触及的设计不作格式化重写。

最后回写 `docs/specs/auto-lang/project.md` 与 `docs/specs/overview.md` 的模块摘要和链接，
运行索引与 lint；`.autoos/specs.json` 仅作为兼容投影按现行 merge/index 流程更新，不能
反向覆盖 canonical Markdown。

## 测试设计

本计划是文档与元数据任务，按 Category A/C 门禁执行，不运行 `cargo t`、`cargo tf` 或
`docs_gen`。产品能力引用既有测试作为证据；只有发现关键断言无法由现有测试支撑时，登记
债务，不在本文档计划中新增测试代码。

最终门禁：

```powershell
python scripts/spec-index.py
python scripts/spec-lint.py --stale-days 7
git diff --check
git status --short
```

另外对已知旧断言做定向扫描，确保它们只出现在历史说明或审计记录中：

```powershell
rg -n "TreeWalker|Evaluator|32-bit|120 opcode|120.*opcode" docs/specs/auto-lang docs/design
```

## 验收标准

- [ ] PLAN-532、PLAN-536 已合并，或用户对未合并项作出明确基线裁定并写入报告。
- [ ] 九个模块均有代码/测试证据、状态分类、漂移裁定和修改清单。
- [ ] 九模块 overview/architecture/design 不再把无实现证据的计划项陈述为 current。
- [ ] interpreter 文档明确公共执行统一走 AutoVM，旧 TreeWalker/Evaluator 只作为历史出现。
- [ ] VM 文档的栈位宽、opcode/指令集和执行路径以当前代码为准；易漂移计数可复现或移除。
- [ ] trans/UI/runtime 对 PLAN-532、PLAN-536 合并后的真实状态有明确证据，不引用在途目标。
- [ ] 顶层 design、`auto-lang/project.md` 与全局 overview 不和 module spec 冲突。
- [ ] 新发现的实现债务已进入 `KNOWN-DEBT-AND-RISKS.md`，没有在文档计划中顺手改代码。
- [ ] `python scripts/spec-index.py` 成功，生成索引无断链。
- [ ] `spec-lint` 不新增 error/warning；既有 warning 有归属说明。
- [ ] `git diff --check` 通过，diff 仅含 docs、spec 投影和必要索引。

## 执行步骤

1. **冻结基线**：在 `docs/reports/module-spec-rebaseline-2026-09-04.md` 记录 `master`
   commit、`git worktree list`、PLAN-532/536 是否已折返；未满足硬前置则停止并写入本计划
   `待澄清事项`。验证：`git branch --show-current`、`git log -1 --oneline`、
   `git worktree list`、`Get-ChildItem docs/plans/archive/532-*.md,docs/plans/archive/536-*.md`；
   仅以归档文件存在和相关提交已进入 `master` 为完成证据，不读取其他计划正文。
   [✅ 已完成] 基线 `170e81ced`；532 worktree 仍在且未进入 master，536 已可从 master
   到达但仍有 worktree 且未归档；证据已写入 rebaseline 报告，按硬门禁停止。
2. **建立报告骨架**：创建九模块统一证据表和复现命令段，不填写无证据结论。验证：
   `rg -n "^## (frontend|types|comptime|interpreter|vm|trans|runtime|ui|mcp)$" docs/reports/module-spec-rebaseline-2026-09-04.md`。
3. **frontend 取证**：核对 `crates/auto-lang/src/{lexer.rs,parser.rs,ast/,resolver.rs,macro_/,dialect/}`
   的模块边界、入口和测试，在报告填写 frontend 裁定。验证：
   `rg -n "pub mod|pub use|#\[test\]" crates/auto-lang/src/lib.rs crates/auto-lang/src/lexer.rs crates/auto-lang/src/parser.rs crates/auto-lang/src/ast crates/auto-lang/src/resolver.rs crates/auto-lang/src/macro_ crates/auto-lang/src/dialect`。
4. **frontend 文档回写**：更新 `docs/specs/auto-lang/frontend/` 中与证据冲突的
   overview/architecture/design/plans。验证：`git diff --check -- docs/specs/auto-lang/frontend`。
5. **types 取证**：核对 infer/typeck/ownership/trait/symbol 实现与测试，在报告填写 types
   裁定。验证：`rg -n "pub mod|pub struct|pub enum|#\[test\]" crates/auto-lang/src/infer* crates/auto-lang/src/type* crates/auto-lang/src/ownership crates/auto-lang/src/trait_checker* crates/auto-lang/src/symbols.rs`。
6. **types 文档回写**：更新 `docs/specs/auto-lang/types/` 中的冲突项。验证：
   `git diff --check -- docs/specs/auto-lang/types`。
7. **comptime 取证**：核对 `crates/auto-lang/src/comptime/`、`ast/comptime.rs` 与
   `compile.rs`，填写 current/experimental/限制。验证：
   `rg -n "pub mod|pub struct|pub enum|#\[test\]" crates/auto-lang/src/comptime crates/auto-lang/src/ast/comptime.rs crates/auto-lang/src/compile.rs`。
8. **comptime 文档回写**：更新 `docs/specs/auto-lang/comptime/`。验证：
   `git diff --check -- docs/specs/auto-lang/comptime`。
9. **interpreter 取证**：沿公共执行入口确认 AutoVM 路径及旧 evaluator 的移除状态，填写
   interpreter 裁定。验证：`rg -n "ExecutionEngine|AutoVM|TreeWalker|Evaluator|evaluate" crates/auto-lang/src`。
10. **interpreter 文档回写**：更新 `docs/specs/auto-lang/interpreter/`，把旧引擎叙事降为
    historical。验证：`git diff --check -- docs/specs/auto-lang/interpreter`。
11. **vm 取证**：从 `vm/opcode.rs`、值/栈定义、builder/engine/loader 与 VM 测试确认指令集、
    栈位宽和执行管线，填写 vm 裁定及可复现计数。验证：
    `rg -n "enum Opcode|type .*Value|pub struct .*Vm|#\[test\]" crates/auto-lang/src/vm crates/auto-lang/src/autovm*.rs crates/auto-lang/src/bigvm*`。
12. **vm 文档回写**：更新 `docs/specs/auto-lang/vm/` 中冲突项，并修正
    `docs/design/05-vm-runtime.md` 的当前态旧断言。验证：
    `git diff --check -- docs/specs/auto-lang/vm docs/design/05-vm-runtime.md`。
13. **trans 取证**：核对 `crates/auto-lang/src/trans/`、各目标 codegen 与 fixture/金样测试，
    填写支持矩阵并区分 current/partial/experimental。验证：
    `rg -n "pub mod|pub struct|Target|#\[test\]" crates/auto-lang/src/trans crates/auto-lang/tests test tests`。
14. **trans 文档回写**：更新 `docs/specs/auto-lang/trans/`，不把尚未落地的 parity 路线写成
    current。验证：`git diff --check -- docs/specs/auto-lang/trans`。
15. **runtime 取证**：核对 scope/session/host/ffi/database 与标准库桥接边界，填写 runtime
    裁定。验证：`rg -n "pub struct|pub enum|pub trait|#\[test\]" crates/auto-lang/src/runtime.rs crates/auto-lang/src/scope.rs crates/auto-lang/src/scope_manager.rs crates/auto-lang/src/session.rs crates/auto-lang/src/host.rs crates/auto-lang/src/ffi.rs crates/auto-lang/src/database`。
16. **runtime 文档回写**：更新 `docs/specs/auto-lang/runtime/`。验证：
    `git diff --check -- docs/specs/auto-lang/runtime`。
17. **ui 取证**：基于 PLAN-536 折返后的 `aura/`、`ui/`、`ui_gen/`、`a2ui/` 与桌面测试，
    填写 Vue/VM/desktop 的真实边界和状态。验证：
    `rg -n "pub mod|pub struct|pub enum|#\[test\]" crates/auto-lang/src/aura crates/auto-lang/src/ui crates/auto-lang/src/ui_gen crates/auto-lang/src/a2ui`。
18. **ui 文档回写**：更新 `docs/specs/auto-lang/ui/`，只修改被代码证据推翻的
    `docs/design/08-ui-systems.md`、`20-autoui-separation-architecture.md` 和
    `docs/design/autoui/` 当前态描述。验证：
    `git diff --check -- docs/specs/auto-lang/ui docs/design/08-ui-systems.md docs/design/20-autoui-separation-architecture.md docs/design/autoui`。
19. **mcp 取证**：核对 server/protocol/session/toolset 入口与测试，填写 mcp 裁定。验证：
    `rg -n "pub mod|pub struct|pub enum|#\[test\]" crates/auto-lang/src/mcp`。
20. **mcp 文档回写**：更新 `docs/specs/auto-lang/mcp/`。验证：
    `git diff --check -- docs/specs/auto-lang/mcp`。
21. **顶层 design 对齐**：按九模块报告定向修正 `docs/design/01-architecture.md`、
    `02-type-system.md`、`03-error-handling.md`、`04-memory-ownership.md`、
    `06-code-generation.md`、`09-compiler.md`、`10-language-syntax.md`、
    `12-concurrency.md`、`13-networking.md`；无证据冲突的文件保持不动。验证：
    `git diff --check -- docs/design`。
22. **入口与债务回写**：更新 `docs/specs/auto-lang/project.md`、`docs/specs/overview.md`；将
    新发现的实现缺口追加到 `docs/plans/KNOWN-DEBT-AND-RISKS.md`，并在报告建立互链。验证：
    `git diff --check -- docs/specs/auto-lang/project.md docs/specs/overview.md docs/plans/KNOWN-DEBT-AND-RISKS.md docs/reports/module-spec-rebaseline-2026-09-04.md`。
23. **生成与最终门禁**：运行 `python scripts/spec-index.py`、
    `python scripts/spec-lint.py --stale-days 7`、已知旧断言扫描和 `git diff --check`；在报告
    记录 warning 基线及每项验收结果。验证：`git status --short` 的修改范围仅限计划允许的
    docs/spec 投影/索引，且未出现 `crates/`、`packages/`、`examples/` 或清单文件。

## 复审记录

待 `/auto-plan:review` 独立复验；不得以本计划中的完成标记代替代码证据。

## 待澄清事项

- **BLOCKED（2026-09-04，Step 1）**：PLAN-532 在
  `D:/autostack/.wt/lang-532/auto-lang`，`plan-532-dev` 尚未进入 `master` 且无归档文件；
  PLAN-536 的分支提交已可从 `master` 到达，但 worktree 仍在且无归档文件。请先完成两者
  的 fold/review/merge，或明确裁定 PLAN-546 排除哪些在途变化；在此之前不执行 Step 2。
