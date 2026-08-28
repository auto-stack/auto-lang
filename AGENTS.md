# AutoLang Agent Guidelines & Workflows

## Standard (Design + Plan + Worktree) Workflow

All AI coding assistants working in this repository must strictly adhere to the following workflow principles based on task complexity.

---

### 1. Task Sizing & Triage

- **L0: Trivial Fixes (轻微修改)**
  - *Criteria*: Minor typos, comment updates, or 1-2 line simple bugfixes without side-effects.
  - *Action*: Directly modify on the current branch, run verification tests, and commit with a clean commit message.
- **L1: Feature / Module Tasks (模块/特性任务)**
  - *Criteria*: Any new feature, multi-file change, complex bugfix, or cross-backend parity implementation.
  - *Action*:
    1. Inspect `docs/plans/.next-id` and existing plans in `docs/plans/` to allocate the next `<NNN>` plan ID.
    2. Create `docs/plans/<NNN>-<plan-name>.md` detailing goals, design, task checklist, and verification plan.
    3. Create a dedicated worktree: `git worktree add .worktree/plan-<NNN> -b plan-<NNN>`.
    4. Perform all implementation and testing inside `.worktree/plan-<NNN>`.
- **L2: Architectural Overhaul (重大架构级任务)**
  - *Criteria*: Changes impacting overall architecture, compiler/VM pipelines, core protocol definitions, or cross-system runtime contracts.
  - *Action*:
    1. First create/update a formal architecture design document in `docs/design/<NN>-<topic>.md`.
    2. Decompose the design into 1 to N moderate-sized, independently executable Plan documents in `docs/plans/<NNN>-*.md`.
    3. Execute each Plan sequentially in its own dedicated worktree.

---

### 2. Execution Discipline in Worktree

- Always perform code modifications, builds, and test runs within `.worktree/plan-<NNN>` to keep `master` clean.
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

Before merging or archiving, the agent **must explicitly execute an independent review step**:
1. **Checklist Audit**: Verify every task in `docs/plans/<NNN>-*.md` is completed without omissions.
2. **Workaround & Debt Scan**: Scan for temporary hacks, hardcoded constants, incomplete TODOs, or workaround patches. Refactor them to adhere cleanly to the architecture.
3. **Health Check**: Ensure zero unhandled compiler warnings, clean formatting, and no stray debug print statements.

---

### 4. Plan Archiving & Status Tracking

1. Update `docs/plans/<NNN>-<plan-name>.md` with final verification results and completion summary.
2. Archive the plan:
   ```bash
   git mv docs/plans/<NNN>-<plan-name>.md docs/plans/archive/
   ```
3. Update `docs/plans/.next-id` with the next available plan ID.

---

### 5. Worktree Merge & Cleanup

1. Switch to `master`, merge the `plan-<NNN>` branch (or cherry-pick/fast-forward) with Conventional Commit format:
   ```bash
   feat(<scope>): <description> (Plan <NNN>)
   ```
2. Remove the temporary worktree and branch:
   ```bash
   git worktree remove .worktree/plan-<NNN>
   git branch -d plan-<NNN>
   ```
3. Summarize the completed work.
