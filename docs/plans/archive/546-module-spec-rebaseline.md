---
plan_id: PLAN-546
status: archived               # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: Auto-Lang Module Spec Rebaseline
author: [Codex]
created_at: 2026-09-04
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/{frontend,types,comptime,interpreter,vm,trans,runtime,mcp} 的 overview/architecture/plans 与部分 design 文件：当前态漂移修正——evaluator 退役叙事对齐、易漂移计数/行号附复现命令与 2026-09-07 快照、伪活跃 plan 行归档化（28 行）、old/→archive/ 路径统一"
  - "docs/specs/auto-lang/ui/plans.md：补录 plan-536 归档行（知识链缺口修复，533/534 直续位）+old/ 清理"
  - "docs/design/{02,03,05,06,08,10,12,13,20}：被模块代码证据推翻的当前态断言修正（推断已接 parser/E0106+auto_type_E020x/194 opcode+NaN-box 64 位/rust.rs 23792/.as·.to·.view 已实现/plan-317 Phase1-4 全完成/stdlib 网络已实现/AURA·api·AutoDown 已实现/RET_D 已删）"
  - "docs/specs/auto-lang/project.md + docs/specs/overview.md：模块清单状态列按各模块 Status 行对齐、mermaid interpreter TreeWalker 节改 AutoVM 外观、Phase-0 骨架脚注换 rebaseline 注记、活跃线 532/536 折返态更新+报告链接"
new_spec_components:
  - "docs/reports/module-spec-rebaseline-2026-09-04.md：九模块代码证据基线报告——基线门禁（532/536 gate+rebase 记录）/九模块九字段证据表/入口回写与债务互链/Step 23 验证日志/验收 recap/提交链"
touched_goals:
  - "GOAL-018: 知识账本九模块 spec 代码证据 rebaseline；曝光归档回写流程缺口（KNOWN-DEBT 546-③）与 plan-536 plans.md 缺席（已补录）"

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
current_step: 23
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

- [x] PLAN-532、PLAN-536 已合并，或用户对未合并项作出明确基线裁定并写入报告。
  （2026-09-06/05 均已折返归档；报告 Baseline gate 节存证，RESOLVED 注记入待澄清事项）
- [x] 九个模块均有代码/测试证据、状态分类、漂移裁定和修改清单。
  （报告九节九字段全填，Steps 3-20 逐模块验证命令命中）
- [x] 九模块 overview/architecture/design 不再把无实现证据的计划项陈述为 current。
  （frontend 325/332/367/448、trans 328/355/364/400/442、runtime 300-458 共 28 行
  伪活跃行校正为已归档；parity 未实施路线均标"已归档"）
- [x] interpreter 文档明确公共执行统一走 AutoVM，旧 TreeWalker/Evaluator 只作为历史出现。
  （复核成立+补强：use-evaluator 空声明精确化、ignore 测试分类）
- [x] VM 文档的栈位宽、opcode/指令集和执行路径以当前代码为准；易漂移计数可复现或移除。
  （NanoValue=u64 NaN-boxing；194 opcode/engine 10139/codegen 14456 行均附 awk/wc
  复现+快照日期；design/05 "~120 opcodes"/"32-bit 栈槽"/RET_D 三处修正）
- [x] trans/UI/runtime 对 PLAN-532、PLAN-536 合并后的真实状态有明确证据，不引用在途目标。
  （trans 行数/用例计数快照复现；ui 补录 plan-536 plans.md 行；runtime 18 行校正；
  overview.md 活跃线 532/536 折返态更新）
- [x] 顶层 design、`auto-lang/project.md` 与全局 overview 不和 module spec 冲突。
  （design 02/03/05/06/08/10/12/13/20 定向修正；project.md mermaid/状态列/脚注；
  overview.md 模块表+活跃线）
- [x] 新发现的实现债务已进入 `KNOWN-DEBT-AND-RISKS.md`，没有在文档计划中顺手改代码。
  （三条 546 债：infer/registry 孤儿、use-evaluator 空声明、归档回写流程缺口；
  零 crates/ 改动——git status 范围检查过）
- [x] `python scripts/spec-index.py` 成功，生成索引无断链。（26 projects）
- [x] `spec-lint` 不新增 error/warning；既有 warning 有归属说明。
  （1 错误预存=plan-577 .next-id 未走 new-plan.sh，master 同报；警告 10→5 净减，
  余 5 条断链归属记录于报告验证日志）
- [x] `git diff --check` 通过，diff 仅含 docs、spec 投影和必要索引。

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
   [✅ 已完成] 九个 `## <module>` 标题全数命中（报告 76-188 行），Evidence method 节含
   九字段表与复现命令；提交 `969503987`。
3. **frontend 取证**：核对 `crates/auto-lang/src/{lexer.rs,parser.rs,ast/,resolver.rs,macro_/,dialect/}`
   的模块边界、入口和测试，在报告填写 frontend 裁定。验证：
   `rg -n "pub mod|pub use|#\[test\]" crates/auto-lang/src/lib.rs crates/auto-lang/src/lexer.rs crates/auto-lang/src/parser.rs crates/auto-lang/src/ast crates/auto-lang/src/resolver.rs crates/auto-lang/src/macro_ crates/auto-lang/src/dialect`。
   [✅ 已完成] 验证命令 399 命中；报告 frontend 节九字段填毕（裁定 5 处漂移：
   evaluator 消费方/行号/13k 规模/活跃 plan 已归档/ADR-06 失效引用）。
4. **frontend 文档回写**：更新 `docs/specs/auto-lang/frontend/` 中与证据冲突的
   overview/architecture/design/plans。验证：`git diff --check -- docs/specs/auto-lang/frontend`。
   [✅ 已完成] overview/architecture/plans 三文件修正，design/ 无证据冲突未动；
   验证通过；提交 `fae421c14`。
5. **types 取证**：核对 infer/typeck/ownership/trait/symbol 实现与测试，在报告填写 types
   裁定。验证：`rg -n "pub mod|pub struct|pub enum|#\[test\]" crates/auto-lang/src/infer* crates/auto-lang/src/type* crates/auto-lang/src/ownership crates/auto-lang/src/trait_checker* crates/auto-lang/src/symbols.rs`。
   [✅ 已完成] 验证命令 194 命中；报告 types 节填毕（Type 39 变体实测、registry
   孤儿化、行号漂移共 5 项裁定；ParamChecker 零调用点复核维持原判）。
6. **types 文档回写**：更新 `docs/specs/auto-lang/types/` 中的冲突项。验证：
   `git diff --check -- docs/specs/auto-lang/types`。
   [✅ 已完成] overview/architecture/plans + design/{type-representation,
   type-inference, typestore} 修正；验证通过；提交 `160322d50`。
7. **comptime 取证**：核对 `crates/auto-lang/src/comptime/`、`ast/comptime.rs` 与
   `compile.rs`，填写 current/experimental/限制。验证：
   `rg -n "pub mod|pub struct|pub enum|#\[test\]" crates/auto-lang/src/comptime crates/auto-lang/src/ast/comptime.rs crates/auto-lang/src/compile.rs`。
   [✅ 已完成] 验证命令 38 命中；七处 CTEE 集成点实测定位；限制性断言全部复核成立。
8. **comptime 文档回写**：更新 `docs/specs/auto-lang/comptime/`。验证：
   `git diff --check -- docs/specs/auto-lang/comptime`。
   [✅ 已完成] 行号漂移 4 处修正+old/→archive/；验证通过；提交 `e2d5d62a0`。
9. **interpreter 取证**：沿公共执行入口确认 AutoVM 路径及旧 evaluator 的移除状态，填写
    interpreter 裁定。验证：`rg -n "ExecutionEngine|AutoVM|TreeWalker|Evaluator|evaluate" crates/auto-lang/src`。
    [✅ 已完成] 验证命令 1508 命中；eval.rs/interp.rs 确认已删（commit 6862bb45f），
    Evaluator 仅重定向 AutoVM；TreeWalker 历史化叙事复核成立。
10. **interpreter 文档回写**：更新 `docs/specs/auto-lang/interpreter/`，把旧引擎叙事降为
    historical。验证：`git diff --check -- docs/specs/auto-lang/interpreter`。
    [✅ 已完成] use-evaluator 空声明精确化、ignore 测试分类细化、行号/old/ 修正；
    验证通过；提交 `f8f15cbd5`。
11. **vm 取证**：从 `vm/opcode.rs`、值/栈定义、builder/engine/loader 与 VM 测试确认指令集、
    栈位宽和执行管线，填写 vm 裁定及可复现计数。验证：
    `rg -n "enum Opcode|type .*Value|pub struct .*Vm|#\[test\]" crates/auto-lang/src/vm crates/auto-lang/src/autovm*.rs crates/auto-lang/src/bigvm*`。
    [✅ 已完成] 验证命令 538 命中；OpCode=194（awk 复现）、栈槽=NanoValue u64
    NaN-boxing（auto-val/nano_value.rs:8）、engine 10139/codegen 14456 行实测。
12. **vm 文档回写**：更新 `docs/specs/auto-lang/vm/` 中冲突项，并修正
    `docs/design/05-vm-runtime.md` 的当前态旧断言。验证：
    `git diff --check -- docs/specs/auto-lang/vm docs/design/05-vm-runtime.md`。
    [✅ 已完成] design/05 三处旧断言（~120 opcode/311 行/32-bit 栈槽/RET_D）+
    RET_D 删除注记；vm spec overview/plans/bytecode-engine 计数同步；验证通过；
    提交 `646ea54b0`。
13. **trans 取证**：核对 `crates/auto-lang/src/trans/`、各目标 codegen 与 fixture/金样测试，
    填写支持矩阵并区分 current/partial/experimental。验证：
    `rg -n "pub mod|pub struct|Target|#\[test\]" crates/auto-lang/src/trans crates/auto-lang/tests test tests`。
    [✅ 已完成] 验证命令 590 命中；八后端+s2s 支持矩阵填毕；行数/用例计数
    漂移量化（rust.rs 13842→23792、a2r 用例 23→262）。
14. **trans 文档回写**：更新 `docs/specs/auto-lang/trans/`，不把尚未落地的 parity 路线写成
    current。验证：`git diff --check -- docs/specs/auto-lang/trans`。
    [✅ 已完成] 规模表/用例计数附复现命令；328/355/364/400/442 归档态修正
    （parity 未实施路线均标"已归档"不称 current）；old/→archive/；提交 `b8e2d6561`。
15. **runtime 取证**：核对 scope/session/host/ffi/database 与标准库桥接边界，填写 runtime
    裁定。验证：`rg -n "pub struct|pub enum|pub trait|#\[test\]" crates/auto-lang/src/runtime.rs crates/auto-lang/src/scope.rs crates/auto-lang/src/scope_manager.rs crates/auto-lang/src/session.rs crates/auto-lang/src/host.rs crates/auto-lang/src/ffi.rs crates/auto-lang/src/database`。
    [✅ 已完成] 验证命令 115 命中；Scope.get_val None 桩/libs/std.rs 0 行/
    stdlib 双文件模式等断言全部复核成立。
16. **runtime 文档回写**：更新 `docs/specs/auto-lang/runtime/`。验证：
    `git diff --check -- docs/specs/auto-lang/runtime`。
    [✅ 已完成] plans.md 18 行伪活跃（标 plans/ 实已归档）整体校正+复核头注；
    old/→archive/；验证通过；提交 `c04c01a1e`。
17. **ui 取证**：基于 PLAN-536 折返后的 `aura/`、`ui/`、`ui_gen/`、`a2ui/` 与桌面测试，
    填写 Vue/VM/desktop 的真实边界和状态。验证：
    `rg -n "pub mod|pub struct|pub enum|#\[test\]" crates/auto-lang/src/aura crates/auto-lang/src/ui crates/auto-lang/src/ui_gen crates/auto-lang/src/a2ui`。
    [✅ 已完成] 验证命令 2629 命中；extract.rs/api//autodown 实现在码实证；
    发现 plan-536 知识链缺口（归档未回写 plans.md）。
18. **ui 文档回写**：更新 `docs/specs/auto-lang/ui/`，只修改被代码证据推翻的
    `docs/design/08-ui-systems.md`、`20-autoui-separation-architecture.md` 和
    `docs/design/autoui/` 当前态描述。验证：
    `git diff --check -- docs/specs/auto-lang/ui docs/design/08-ui-systems.md docs/design/20-autoui-separation-architecture.md docs/design/autoui`。
    [✅ 已完成] ui/plans.md 补录 536 行+old/ 清理；design/08 四处现状修正；
    design/20 失效路径修正（autoui/ 零冲突）；验证通过；提交 `068054dea`。
19. **mcp 取证**：核对 server/protocol/session/toolset 入口与测试，填写 mcp 裁定。验证：
    `rg -n "pub mod|pub struct|pub enum|#\[test\]" crates/auto-lang/src/mcp`。
    [✅ 已完成] 验证命令命中（5 文件 838 行）；7 工具/边界声明/坑清单
    （sandbox 空标志、typecheck 名不副实、无 GC）全部复核成立。
20. **mcp 文档回写**：更新 `docs/specs/auto-lang/mcp/`。验证：
    `git diff --check -- docs/specs/auto-lang/mcp`。
    [✅ 已完成] CLI 行号 1547→2010 附快照；old/→archive/；验证通过；
    提交 `a09670a12`。
21. **顶层 design 对齐**：按九模块报告定向修正 `docs/design/01-architecture.md`、
    `02-type-system.md`、`03-error-handling.md`、`04-memory-ownership.md`、
    `06-code-generation.md`、`09-compiler.md`、`10-language-syntax.md`、
    `12-concurrency.md`、`13-networking.md`；无证据冲突的文件保持不动。验证：
    `git diff --check -- docs/design`。
    [✅ 已完成] 修正 02（推断已接入 parser+内联约束）、03（E0106+E020x 前缀）、
    06（rust.rs 23792）、10（.as/.to/.view/.mut/.move 已实现）、12（317
    Phase 1-4 全完成+scheduler 在产）、13（stdlib 网络模块已实现）；01/04/09
    无冲突未动；另修 vm overview+runtime plans 的 plan-317 过时状态；提交 `d97f52f27`。
21. **顶层 design 对齐**：按九模块报告定向修正 `docs/design/01-architecture.md`、
    `02-type-system.md`、`03-error-handling.md`、`04-memory-ownership.md`、
    `06-code-generation.md`、`09-compiler.md`、`10-language-syntax.md`、
    `12-concurrency.md`、`13-networking.md`；无证据冲突的文件保持不动。验证：
    `git diff --check -- docs/design`。
22. **入口与债务回写**：更新 `docs/specs/auto-lang/project.md`、`docs/specs/overview.md`；将
    新发现的实现缺口追加到 `docs/plans/KNOWN-DEBT-AND-RISKS.md`，并在报告建立互链。验证：
    `git diff --check -- docs/specs/auto-lang/project.md docs/specs/overview.md docs/plans/KNOWN-DEBT-AND-RISKS.md docs/reports/module-spec-rebaseline-2026-09-04.md`。
    [✅ 已完成] project.md mermaid TreeWalker 节/状态列/骨架脚注修正；overview.md
    532/536 折返态+报告链接；KNOWN-DEBT 登记三条 546 债（infer/registry 孤儿/
    use-evaluator 空声明/归档回写流程缺口）；报告互链节；提交 `3a07b0856`。
23. **生成与最终门禁**：运行 `python scripts/spec-index.py`、
    `python scripts/spec-lint.py --stale-days 7`、已知旧断言扫描和 `git diff --check`；在报告
    记录 warning 基线及每项验收结果。验证：`git status --short` 的修改范围仅限计划允许的
    docs/spec 投影/索引，且未出现 `crates/`、`packages/`、`examples/` 或清单文件。
    [✅ 已完成] INDEX 再生 26 projects；lint 1 错误预存（.next-id 575<577，
    plan-577 立项未走 new-plan.sh，master 同报）+警告 10→5 净减（五个 stale
    overview 被本计划刷新消除）；旧断言余量全为合法历史语境；diff --check 过；
    范围干净（零 crates/packages/examples/清单）；验证日志+验收 recap 入报告；
    提交 `ee09a2c28`。

## 复审记录

**复审通过（2026-09-07，reviewer：ZCode 独立复验轮）**——验收 11/11 PASS，零阻断债，`status: reviewed`。

**复审方法**：不信任执行期勾选，全部关键判据在 worktree（`D:/autostack/.wt/lang-546/auto-lang`，`plan-546-dev`，base `1c6753a92`，15 提交）独立重跑/抽查：

1. **diff 范围**：42 文件全 `docs/`（+673/−361），零 `crates/`/`packages/`/`examples/`/清单——Category A 成立；工作区干净。
2. **四项终门禁复跑**：spec-index 26 projects 无断链 ✅；spec-lint 1 错误+5 警告——错误（`.next-id=575`<577）master 同报预存（plan-577 会话未走 new-plan.sh，非本计划引入），警告较 master 10→5 净减（五个 stale overview 被刷新消除）且余量全归属 ✅；旧断言扫描余量全为合法历史语境（ADR/raw 归档/NaN-box payload 位域描述）✅；`git diff --check` 过 ✅。
3. **承重结论抽查 8/8**：OpCode=194（awk 重跑）、`NanoValue=u64`（nano_value.rs:8）、rust.rs=23792（wc 重跑）、Type=39 变体、plan-317"Phase 1-4 全部完成"出自归档 plan 头部核查回填（非转述）、ui/plans.md 536 行 1 命中、KNOWN-DEBT `| 546 |` 3 行、532/536 归档文件双在。
4. **全量测试门禁裁定**：本计划按其测试设计为 Category A/C（纯文档/元数据），AGENTS 门禁明文禁止 docs-only 任务跑 `cargo t/tf/docs_gen`；复审以计划自身四项门禁+42 文件 docs-only diff 佐证替代 cargo 全量——合法且充分。

**逐项验收判定**：

| # | 验收项 | 判定 | 证据 |
|---|---|---|---|
| 1 | 532/536 已合并 | PASS | archive 双文件在；`merge(plan532)`=d4fe4ae48 可达；报告 Baseline gate 节 |
| 2 | 九模块证据表 | PASS | 报告 9 节 9 字段（rg 计数 9）；每节含漂移裁定+修改清单 |
| 3 | 计划项不冒充 current | PASS | 28 伪活跃行校正（frontend 4/trans 5/runtime 18+行内）；parity 未实施线均标"已归档" |
| 4 | interpreter 统一 AutoVM | PASS | overview/architecture 复核成立；execution_engine.rs:3 头注佐证 |
| 5 | VM 位宽/指令集可复现 | PASS | awk 194/wc 10139/14456 复现命令入文；design/05 三处修正 |
| 6 | trans/UI/runtime post-532/536 证据 | PASS | trans 计数快照；ui 536 行补录；runtime 18 行校正 |
| 7 | 顶层 design/project/overview 不冲突 | PASS | design 9 文件定向修正；project mermaid/状态列；overview 活跃线 |
| 8 | 债务入账、零顺手改码 | PASS | KNOWN-DEBT 3 行；diff 零非 docs 文件 |
| 9 | spec-index 无断链 | PASS | 复跑 26 projects |
| 10 | lint 零新增+归属 | PASS | 复跑比对 master：错误同、警告净减 5、余量归属入报告 |
| 11 | diff --check+范围 | PASS | 复跑过；42 文件全 docs |

**遗漏/延后/workaround 猎查**：零阻断项。两条非阻断备忘——
- N1（预存断链族，已归属）：ui/overview.md sidebar 链接相对深度错（`../../`应为`../../../`）+aavm/project.md→523/525/536 归档路径+goals.md→autoshell.md 缺文件，共 5 警告；aavm/goals 在九模块范围外，ui 链接与 GOAL-018 入口链精神相关但非本计划步骤——统归流程债 546-③（KNOWN-DEBT 在案），留后续链接族清理批。
- N2（报告完备性微瑕）：报告未显式记录"design 01/04/09 已查无冲突"（该结论仅在计划步骤标记中）；复审已补验 design/04（Status 节 ParamMode 三元/Hold partial/计划项属实，零冲突）与 design/01（Evaluator 已历史化），不改判。

**spec-impact 元数据**已填（frontmatter）：supersedes 4 条（九模块 spec 修正/ui 536 补录/design 9 文件/入口两文件）、new 1 条（rebaseline 报告）、touched_goals=GOAL-018。

**路由**：全部通过 → `status: reviewed`，就绪待 `/auto-plan:merge`。

## 待澄清事项

- **BLOCKED（2026-09-04，Step 1）**：PLAN-532 在
  `D:/autostack/.wt/lang-532/auto-lang`，`plan-532-dev` 尚未进入 `master` 且无归档文件；
  PLAN-536 的分支提交已可从 `master` 到达，但 worktree 仍在且无归档文件。请先完成两者
  的 fold/review/merge，或明确裁定 PLAN-546 排除哪些在途变化；在此之前不执行 Step 2。
  - **RESOLVED（2026-09-07）**：硬前置已满足——532 归档文件在
    （`merge(plan532)` = `d4fe4ae48` 进入 `master`）、536 归档文件在（复审 + T12 修复
    均可达 `master`），两计划 worktree/分支已清。worktree 分支 rebase 到当前
    `master`（`1c6753a92`；旧基线 `170e81ced` 因 master 历史重写已不在祖先链），
    基线重冻结记录见报告 Baseline snapshot 节，自 Step 2 恢复执行。
