# Auto-Lang Module Spec Rebaseline Report

> PLAN-546 execution evidence. Baseline gate passed on resume (2026-09-07).

## Baseline snapshot

- First freeze: 2026-09-04 (Asia/Shanghai) at `170e81ced docs:archive-plan-543` — gate blocked, see below.
- Resume freeze: 2026-09-07 (Asia/Shanghai) at `1c6753a92 merge(plan534)`.
- Execution branch: `plan-546-dev` (worktree `D:/autostack/.wt/lang-546/auto-lang`).
- Default branch: `master`.
- History note: `170e81ced` is no longer an ancestor of `master` (master's line was
  rewritten after the first freeze). The branch was rebased onto current `master`
  on resume; the only branch-unique commit (the original gate record) was
  preserved. All evidence below is gathered against the post-rebase tree.

Reproduction commands:

```powershell
git branch --show-current
git log -1 --oneline
git worktree list
Test-Path 'docs\plans\archive\532-*.md'   # True
Test-Path 'docs\plans\archive\536-*.md'   # True
git branch --merged master                 # 532/536 branches deleted after fold
git log --oneline master --grep=532        # merge(plan532) d4fe4ae48 reachable
git log --oneline master --grep=536        # plan536 review + T12 fix reachable
```

## PLAN-532 / PLAN-536 gate

| Plan | Archived file | Reachable from `master` | Worktree / branch residue | Gate result |
|---|---:|---:|---|---|
| PLAN-532 | `docs/plans/archive/532-aavm-tower-selfhost.md` | yes (`merge(plan532)` = `d4fe4ae48`) | none (cleaned) | pass |
| PLAN-536 | `docs/plans/archive/536-vm-reactive-runtime-fixes.md` | yes (review + `fix(vm)` T12 commits in history) | none (cleaned) | pass |

First-freeze state (2026-09-04, for the record): PLAN-532 worktree existed at
`066d12493` with `plan-532-dev` unmerged and no archive file; PLAN-536 commits
were reachable but the worktree existed and no archive file was present. Both
blocked the gate. On 2026-09-07 both plans are folded, archived, and cleaned —
the hard prerequisite of PLAN-546 §需求分析 is satisfied and execution resumes
from Step 2 on the post-532/post-536 `master`.

## Decision

Baseline accepted at `1c6753a92`. Module evidence gathering (Steps 3-20) proceeds
on this checkout. This plan changes knowledge documents and generated indexes
only — no product code.

## Evidence method

Each module section records the same nine fields, filled only from code/test
evidence gathered at `1c6753a92` (or the worktree HEAD above it):

| Field | Meaning |
|---|---|
| 边界 (boundary) | Files/dirs that constitute the module; what is explicitly outside |
| 生产入口 (production entries) | Public functions/structs reachable from real execution paths (not only tests) |
| 公共 API / 关键数据流 | Key public types and the dataflow between them |
| 当前能力 (current) | Capabilities with production entry and/or a runnable test as evidence |
| 实验能力 (experimental) | Gated/incomplete paths that exist in code but lack full evidence |
| 未实现/计划项 (planned) | Design/plan goals with no code — must never be stated as current |
| 测试证据 (tests) | Concrete test modules/files and how to run them |
| 与旧文档的差异 (drift) | Assertions in current specs found unsupported/contradicted by code |
| 本次修改文件 (files changed) | Spec/design files edited in this rebaseline |

Reproduction commands (run from repo root, in the plan worktree):

```powershell
rg -n "pub mod|pub use|#\[test\]" crates/auto-lang/src/lib.rs
cargo check -p auto-lang        # structural sanity only; this plan changes no code
rg -n "TreeWalker|Evaluator|32-bit|120 opcode|120.*opcode" docs/specs/auto-lang docs/design
```

Status vocabulary: `current` / `experimental` / `planned` / `historical` (PLAN-546 校准规则).

## frontend

| Field | Evidence |
|---|---|
| 边界 | `crates/auto-lang/src/{lexer,token,parser,parser_helpers,ast,ast/,dialect,dialect/,use_scanner,resolver}.rs`；`macro_/`（UI 宏）与 `mode.rs` 脚本方言判定相邻。执行/类型检查/转译在边界外 |
| 生产入口 | `lib.rs` `pub fn parse`（快照 2490 行）；`Parser`（`Parser::parse/parse_stmt/parse_expr/build_dialects/try_dialect_stmt`）；`use_scanner::scan_use_statements`；`resolver::{ModuleResolver, FilesystemResolver}`；`dialect::Dialect` trait + `UiDialect`；`mode.rs::resolve_script_mode`（plan-555 八格矩阵） |
| 公共 API / 关键数据流 | 源码 → `Lexer`（内部模块，f-string `$var`/`${expr}` 插值，`fstr_note '$'`）→ `TokenKind/Token/Pos` → `Parser`（持 `TypeStore`/`InferenceContext`/`ModuleTracker`/方言表）→ `Code/Stmt/Expr`（`ast.rs` + `ast/` 33 文件）→ `ToNode/ToAtom/AtomWriter` 序列化（166 impls，`rg -c` ast.rs+ast/*.rs） |
| 当前能力 | 递归下降全语法解析；方言派发（`Dialect` trait，UiDialect 接管 widget/msg/model/view/on，mem::take 模式 611-634 行）；`.as/.to` Cast/To（2714/3027/3037 行）；`super/pac` ModulePath 解析；use 两层扫描（字符串级 + ModuleResolver）；W2 语法糖批（plan-560：with/Power `**`/`@`/`is` 中缀）；with-as 绑定（plan-567，`with_header` pratt 截断）；ScriptMode 八格矩阵（plan-555） |
| 实验能力 | s2s lower 管线经 `auto_s2s` 消费 parser 产物（`.as` 执行翻转 lower→compile，plan-560——成熟度归 trans 节裁定） |
| 未实现/计划项 | 符号属性 `.?`/`.*`/`.@`、位操作、Auto Flow `Iter<T>`（ADR-08 设计层 active、实现 planned）——overview Status 行已如实标注 |
| 测试证据 | 内联 `mod tests`：lexer.rs/parser.rs/token.rs/resolver.rs（`rg -l "mod tests"`）；`src/tests/{test_generic_parse,widget_macro_tests,conformance_tests}.rs` 等；plan550/555/560/567 验收测试在对应归档 plan 有账 |
| 与旧文档的差异 | ①overview/architecture "求值器…四类后端消费/后端：evaluator"——Evaluator 仅重定向 AutoVM（execution_engine.rs:3 头注 Plan 091），已改述；②行号断言漂移（parser.rs 1979/2257→2714+/3027+、lib.rs 2114→2490、mem::take 434→611），已附 rg 复现+快照日期；③"parser.rs 超 13k 行"→实测 20562 行；④plans.md "活跃 plan" 325/332/367/448 实均已归档（status 以归档文件为准），`old/` 归档引用已统一为 `archive/`；⑤ADR-06 引 parser.rs:188 注释已失效（现为 Plan 306 GDScript 注解），改引 dialect.rs |
| 本次修改文件 | `docs/specs/auto-lang/frontend/{overview,architecture,plans}.md`（design/ 四文件无证据冲突，未动） |

## types

| Field | Evidence |
|---|---|
| 边界 | 待取证 |
| 生产入口 | 待取证 |
| 公共 API / 关键数据流 | 待取证 |
| 当前能力 | 待取证 |
| 实验能力 | 待取证 |
| 未实现/计划项 | 待取证 |
| 测试证据 | 待取证 |
| 与旧文档的差异 | 待取证 |
| 本次修改文件 | 待取证 |

## comptime

| Field | Evidence |
|---|---|
| 边界 | 待取证 |
| 生产入口 | 待取证 |
| 公共 API / 关键数据流 | 待取证 |
| 当前能力 | 待取证 |
| 实验能力 | 待取证 |
| 未实现/计划项 | 待取证 |
| 测试证据 | 待取证 |
| 与旧文档的差异 | 待取证 |
| 本次修改文件 | 待取证 |

## interpreter

| Field | Evidence |
|---|---|
| 边界 | 待取证 |
| 生产入口 | 待取证 |
| 公共 API / 关键数据流 | 待取证 |
| 当前能力 | 待取证 |
| 实验能力 | 待取证 |
| 未实现/计划项 | 待取证 |
| 测试证据 | 待取证 |
| 与旧文档的差异 | 待取证 |
| 本次修改文件 | 待取证 |

## vm

| Field | Evidence |
|---|---|
| 边界 | 待取证 |
| 生产入口 | 待取证 |
| 公共 API / 关键数据流 | 待取证 |
| 当前能力 | 待取证 |
| 实验能力 | 待取证 |
| 未实现/计划项 | 待取证 |
| 测试证据 | 待取证 |
| 与旧文档的差异 | 待取证 |
| 本次修改文件 | 待取证 |

## trans

| Field | Evidence |
|---|---|
| 边界 | 待取证 |
| 生产入口 | 待取证 |
| 公共 API / 关键数据流 | 待取证 |
| 当前能力 | 待取证 |
| 实验能力 | 待取证 |
| 未实现/计划项 | 待取证 |
| 测试证据 | 待取证 |
| 与旧文档的差异 | 待取证 |
| 本次修改文件 | 待取证 |

## runtime

| Field | Evidence |
|---|---|
| 边界 | 待取证 |
| 生产入口 | 待取证 |
| 公共 API / 关键数据流 | 待取证 |
| 当前能力 | 待取证 |
| 实验能力 | 待取证 |
| 未实现/计划项 | 待取证 |
| 测试证据 | 待取证 |
| 与旧文档的差异 | 待取证 |
| 本次修改文件 | 待取证 |

## ui

| Field | Evidence |
|---|---|
| 边界 | 待取证 |
| 生产入口 | 待取证 |
| 公共 API / 关键数据流 | 待取证 |
| 当前能力 | 待取证 |
| 实验能力 | 待取证 |
| 未实现/计划项 | 待取证 |
| 测试证据 | 待取证 |
| 与旧文档的差异 | 待取证 |
| 本次修改文件 | 待取证 |

## mcp

| Field | Evidence |
|---|---|
| 边界 | 待取证 |
| 生产入口 | 待取证 |
| 公共 API / 关键数据流 | 待取证 |
| 当前能力 | 待取证 |
| 实验能力 | 待取证 |
| 未实现/计划项 | 待取证 |
| 测试证据 | 待取证 |
| 与旧文档的差异 | 待取证 |
| 本次修改文件 | 待取证 |
