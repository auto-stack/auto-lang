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
| 边界 | 待取证 |
| 生产入口 | 待取证 |
| 公共 API / 关键数据流 | 待取证 |
| 当前能力 | 待取证 |
| 实验能力 | 待取证 |
| 未实现/计划项 | 待取证 |
| 测试证据 | 待取证 |
| 与旧文档的差异 | 待取证 |
| 本次修改文件 | 待取证 |

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
