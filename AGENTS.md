# AutoLang Agent Guidelines & Workflows

## Standard (Design + Plan + Worktree) Workflow

All AI coding assistants working in this repository must strictly adhere to the following workflow principles based on task complexity.

> **开发范式（2026-08-28 起，Plan 467）**：标准流程采用 **auto-plan 四技能范式**
> （`/auto-plan:new` → `/auto-plan:work` → `/auto-plan:review` → `/auto-plan:merge`），
> 范式设计与规约见 [docs/design/autoplan-spec-ledger.md](docs/design/autoplan-spec-ledger.md)
> 与 [docs/specs/README.md](docs/specs/README.md) §4。**路径映射**（四技能为 auto-os 仓书写，
> 在本仓执行时以此为准）：技能文档中的 `docs/plans/archived/` 在本仓为 **`docs/plans/archive/`**；
> 技能文档中的 `.worktrees/plan-<NNN>-dev` 在本仓为 **`D:/autostack/.wt/lang-<NNN>/auto-lang`**
> （Plan 529 分组平铺布局；在途的 525/526 仍用旧 `.worktrees/`，fold 后退役）；
> `.autoos/specs.json` 两仓一致。
> Plan 状态机：`drafting → executing → execution_done → reviewed → archived`（终态）。
>
> **Worktree 红线（2026-09-03 三仓 .git 删除事故，Plan 529）**：worktree 内**禁止创建任何
> junction/symlink**——`git worktree remove` 的递归删除会穿透链接删除目标仓内容（实测复现）。
> 跨仓依赖用解析序解决：`$AUTO_LANG_ROOT 等 env 覆盖 → 组内 ../auto-lang →
> D:/autostack/auto-lang 主检出`。移除任何 worktree 前必须先跑
> `bash D:/autostack/wt-guard.sh <worktree 路径>`（reparse point 扫描，非空即拒）。

---

### 1. Task Sizing & Triage

- **L0: Trivial Fixes (轻微修改)**
  - *Criteria*: Minor typos, comment updates, or 1-2 line simple bugfixes without side-effects.
  - *Action*: Directly modify on the current branch, run verification tests, and commit with a clean commit message.
- **L1: Feature / Module Tasks (模块/特性任务)**
  - *Criteria*: Any new feature, multi-file change, complex bugfix, or cross-backend parity implementation.
  - *Action*:
    1. Run `scripts/new-plan.sh <slug>` on the default checkout (master) to atomically take the next `<NNN>` plan ID from `docs/plans/.next-id` and create the plan skeleton (v2 frontmatter).
    2. Fill `docs/plans/<NNN>-<plan-name>.md` (needs-analysis seeded from [docs/specs/overview.md](docs/specs/overview.md)) detailing goals, design, task checklist, and verification plan; present for confirmation before executing.
    3. Create a dedicated worktree in the sibling-group layout (Plan 529): `git worktree add D:/autostack/.wt/lang-<NNN>/auto-lang -b plan-<NNN>-dev`. Cross-repo plans add sibling worktrees into the same group dir (e.g. `.wt/down-047/{auto-down, auto-lang}`) so `../auto-lang` resolves uniformly.
    4. Perform all code implementation and testing inside that worktree; plan-file bookkeeping (`[✅]` markers, frontmatter flips) stays on the default checkout.
- **L2: Architectural Overhaul (重大架构级任务)**
  - *Criteria*: Changes impacting overall architecture, compiler/VM pipelines, core protocol definitions, or cross-system runtime contracts.
  - *Action*:
    1. First create/update a formal architecture design document in `docs/design/<NN>-<topic>.md` (register it in `docs/design/00-intro.md`).
    2. Decompose the design into 1 to N moderate-sized, independently executable Plan documents in `docs/plans/<NNN>-*.md`.
    3. Execute each Plan sequentially in its own dedicated worktree (one worktree per plan for its whole lifetime).

---

### 2. Execution Discipline in Worktree

- Always perform code modifications, builds, and test runs within the plan's worktree (`D:/autostack/.wt/lang-<NNN>/auto-lang`; legacy in-flight plans keep their `.worktrees/` path until folded) to keep `master` clean (one worktree per plan per repo; multi-phase plans fold per phase and re-sync).
- **Never place junctions/symlinks inside a worktree** (see red line above); resolve cross-repo deps via the documented order (env → group sibling → main checkout), never via links.
- **Fast Iteration during Development**:
  - During development, **DO NOT run full test suites repeatedly**. Use fast syntax/type checks:
    - Fast type check: `cargo check -p auto-lang`
    - Scoped single-module test (optional): `cargo t <module_name>` (e.g. `cargo t iced`)
  - Reuse standardized automation scripts from `.agents/skills/autoui-verifier/scripts/` (`test_vm_mcp.py`, `test_vue_playwright.mjs`) instead of writing ad-hoc scripts.
- **Change-Scoped Verification Gate (按改动范围分级测试门禁)**:
  - **Category A: Pure Verification / Asset / Doc-Only Tasks (纯验证/示例资产/计划跟踪)**:
    - 若未修改 `crates/` 下的 Rust 源码（仅截图、更新 `docs/plans/` 矩阵或修改测试脚本），**严禁运行 `cargo t` 和 `docs_gen`**。仅完成目标验证即可直接合入。
  - **Category B: Scoped Rust Code Changes (局部 Rust 模块改动)**:
    - 快速语法/类型检查：`cargo check -p auto-lang`
    - 局部模块验证：`cargo t <module_name>`（如 `cargo t iced` 或 `cargo t ui`）
    - 涉及编译器/VM/核心协议重构时，才在最终合入前运行一次 `cargo tf`（full 档，含 1M churn；Plan 466）。
  - **Category C: Docs / Schema Changes (文档与元数据改动)**:
    - **仅当**修改了文档生成器、Schema 定义文件或语法参考时，才运行 `cargo test -p auto-lang --test docs_gen`。
  - **AutoUI 跨端验证（双端模式）**:
    - Vue 模式：`auto run`
    - VM 模式：`auto run -r vm`
    - 自动化双端一致性：调用 `autoui-verifier` 技能 (`.agents/skills/autoui-verifier`)。

#### Cargo Test Aliases Reference (from `.cargo/config.toml`)
- `cargo t`  - Fast daily tests (~3200 unit tests via nextest in parallel; 1M churn tier excluded, Plan 466)
- `cargo tf` - Full-scale daily tests (all tests incl. 1M churn tier) — the review / pre-fold full-suite gate (Plan 466)
- `cargo tv` - VM file tests (`--features test-vm-files`)
- `cargo tt` - Transpiler tests (`--features test-trans`)
- `cargo tb` - Book listing tests (`--features test-book`)
- `cargo ta` - All test suites combined (`--features test-vm-files,test-trans,test-book`; full scale)


---

### 3. Mandatory Independent Review Gate (独立复审)

Before merging or archiving, the agent **must explicitly execute an independent review step** (`/auto-plan:review` paradigm — verify, don't trust):
1. **Checklist Audit**: Re-verify every acceptance criterion in `docs/plans/<NNN>-*.md` against the actual code/diff — a checked box is a claim, not evidence.
2. **遗漏/延后/Workaround Scan**: Hunt explicitly for dropped sub-items, unapproved deferrals, and workaround patches; record findings as debt candidates in `docs/plans/KNOWN-DEBT-AND-RISKS.md`.
3. **Health Check**: Ensure zero unhandled compiler warnings, clean formatting, and no stray debug print statements.
4. Fill the spec-impact metadata (`supersedes_spec_components` / `new_spec_components` / `touched_goals`) so merge knows what to deposit.

---

### 4. Plan Archiving & Status Tracking

1. Update `docs/plans/<NNN>-<plan-name>.md` with the review record (复审记录) and completion summary; flip `status: reviewed`.
2. Deposit knowledge per `/auto-plan:merge` + [docs/specs/README.md](docs/specs/README.md) §4 extension (`.autoos/specs.json` upsert + module overview/ADR/plans.md 回写 + `python scripts/spec-index.py`).
3. Archive the plan (terminal state — archived plans do not go back):
   ```bash
   git mv docs/plans/<NNN>-<plan-name>.md docs/plans/archive/
   ```
   then set `status: archived` in its frontmatter. **Note:** the user-level auto-plan skills write `docs/plans/archived/` — in THIS repo the archive directory is `docs/plans/archive/`.
4. `docs/plans/.next-id` is bumped atomically by `scripts/new-plan.sh` at plan creation time.

---

### 5. Worktree Merge & Cleanup

1. Switch to `master`, merge the `plan-<NNN>-dev` branch (or cherry-pick/fast-forward) with Conventional Commit format:
   ```bash
   feat(<scope>): <description> (Plan <NNN>)
   ```
2. Remove the temporary worktree and branch (guard first — mandatory):
   ```bash
   bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-<NNN>/auto-lang   # 必须输出 clean 才继续
   git worktree remove D:/autostack/.wt/lang-<NNN>/auto-lang
   git branch -d plan-<NNN>-dev
   # 组内已无兄弟 worktree 时删除组目录：rmdir D:/autostack/.wt/lang-<NNN>
   ```
3. Summarize the completed work.
