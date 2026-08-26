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
- Ensure all relevant test suites pass:
  - Unit tests: `cargo test -p auto-lang --lib`
  - Docs & schema consistency: `cargo test -p auto-lang --test docs_gen`
  - End-to-end / multi-backend: `auto run` (Vue) vs `auto run -r vm` (Iced), `auto build --gen-only`.

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
