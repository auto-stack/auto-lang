---
name: auto-plan-work
description: |
  Execute an implementation plan step by step, using the plan file as the sole
  context. Walks the task list, marks each step done, advances the status
  (drafting → executing → execution_done), and logs blockers to the plan's
  open-questions section instead of going off-script. Use when:
  (1) User says "execute plan N" / "执行 plan N" / "继续做 plan" / "work on plan 42"
  (2) User says "/auto-plan:work" or "按这个 plan 干活"
  (3) A plan in docs/plans/ is in drafting/executing status and the user wants to advance it
  Reads only the target plan — never loads specs or other plans mid-execution.
  All code changes happen in a dedicated git worktree `.worktrees/plan-<NNN>-dev`
  (never on the default checkout); plan-file progress markers stay on the
  default checkout so every skill can see them.
---

# /auto-plan:work — Execute a plan

Execute one plan, end to end, treating that plan as the **only** context. One
skill, one session, one plan. Resumable — if interrupted, re-run and resume from
the first unchecked task.

> **Design source:** `docs/designs/008-auto-plan.md` §6.3.

**Announce at start:** "I'm using /auto-plan:work to execute plan `<NNN>`."

**Input:** A plan reference — a number (`42`), a filename (`042-*.md`), or the
literal "执行 Plan N". If omitted, pick the newest `drafting` or `executing`
plan in `docs/plans/`.

**State gate:** The plan must be `drafting` or `executing`. If it is
`execution_done` / `reviewed` / `archived`, refuse and point the user to the
right next skill (`/auto-plan:review` for `execution_done`).

## Process

### Step 1: Locate and load the target plan

```bash
# By number — match the 3-digit prefix in active OR archived dir:
ls docs/plans/<NNN>-*.md docs/plans/archived/<NNN>-*.md 2>/dev/null
```

Read that one file. It is now the sole context for this session. **Do not load
specs, other plans, or the design doc** — the plan already contains everything
needed; going off-script is how steps get dropped (008 §6.3 constraint).

### Step 2: Set up the execution worktree BEFORE touching any code

All modification of this repo happens inside a dedicated git worktree — never
directly on the default checkout. Worktrees always live under the project
root's `.worktrees/` directory:

```bash
# From the default checkout. Resume-safe: reuse if it already exists.
git worktree list | grep -q plan-<NNN>-dev || \
  git worktree add .worktrees/plan-<NNN>-dev -b plan-<NNN>-dev
cd .worktrees/plan-<NNN>-dev
```

- Branch name = worktree name (`plan-<NNN>-dev`). Commit completed steps onto
  that branch as you go. First entry is cold — install deps / rebuild inside
  the worktree as the plan's steps require.
- **ONE worktree per plan per repo — for the plan's whole lifetime.** Never
  open a second worktree for a later phase/batch/concern of the same plan;
  later phases commit onto the same branch. If you find yourself typing
  `git worktree add` mid-plan, that is a process bug — reuse the existing
  worktree.
- **Multi-phase plans land incrementally.** After a phase's verification
  passes (and its commits are in), merge the branch into the default
  branch, then sync the default branch back into the worktree
  (`git merge <default>` inside the worktree) before starting the next
  phase. Long-running plans drift badly behind a live default branch
  (parallel sessions keep advancing it); per-phase fold + re-sync keeps the
  worktree current and lets cross-repo consumers pick landed phases up
  early. The worktree itself stays put — final cleanup/removal remains
  `/auto-plan:merge`'s job.
- **Plan-file bookkeeping stays on the default checkout.** `[✅]` markers,
  frontmatter flips, and 待澄清事项 entries go into the main checkout's
  `docs/plans/<NNN>-*.md` — every skill reads the plan from there, so progress
  must stay visible on the default checkout. Only product/code changes belong
  in the worktree.
- **Dependency projects:** when a step must modify another project this repo
  depends on (e.g. `auto-musk` depends on `auto-lang`), open ONE worktree in
  THAT project — named after THIS project: `<dep-root>/.worktrees/auto-musk-dev`,
  same-name branch, reused for the whole plan (same one-worktree rule).
  Never edit a dependency checkout outside its own worktree. Fold each
  dependency worktree back into the dependency's main branch as soon
  as this repo consumes the change (dependency bump verified in integration) —
  don't leave them dangling until plan completion.

### Step 3: Advance the state machine if needed

If `status: drafting`, flip it to `executing` in the frontmatter and bump
`updated_at`. If already `executing`, leave it. Legal transitions:
`drafting → executing`; `executing → execution_done` (Step 6).

### Step 4: Execute tasks in order, marking each done

Work through `## 执行步骤` top-to-bottom. For each step:

1. Do exactly what the step says — precise file path, exact operation.
2. Run the step's verification command. It must pass before moving on.
3. Append `[✅ 已完成] <one-line evidence>` next to the step in the plan file.
4. Bump `current_step` in the frontmatter.

**TDD order (superpowers rule):** when a step writes code with tests, write the
failing test first, confirm it fails, then implement, then confirm it passes.

### Step 5: Log blockers in-plan, do NOT improvise

When a step is ambiguous or blocked, **do not** go search specs or other docs to
figure it out. Instead append a bullet under `## 待澄清事项` (Open Questions)
and stop there for the user to resolve. Going off-script to "research" is how
execution drifts from the reviewed plan.

### Step 6: When all tasks are done → execution_done

Once every step has `[✅]` and every verification has passed:

1. Set `status: execution_done` in the frontmatter.
2. Run the plan's whole verification suite (acceptance criteria section) once more —
   inside the worktree.
3. Hand off: tell the user the plan is ready for `/auto-plan:review`.

Leave `.worktrees/plan-<NNN>-dev` (and its branch) in place — final fold +
cleanup + deletion is `/auto-plan:merge`'s job. (Per-phase incremental merges
into the default branch, per Step 2, are landing progress — they are not the
terminal fold and do not remove the worktree.) Do not run the review or merge
skills — those are separate hand-offs.

## When to stop and ask for help

Stop immediately and surface to the user if:
- A verification command fails repeatedly and the fix is unclear.
- A step's instructions contradict the current code and you cannot tell which is right.
- The plan references a file/path/module that does not exist and the intent is unclear.

Ask rather than guess — a wrong step propagates to every later step.

## Rules

- **Only read the target plan.** No specs, no other plans, no design docs mid-flight.
- **One worktree per plan per repo, whole plan lifetime** — no per-phase or
  per-concern worktrees; multi-phase plans land by merging the branch into
  the default branch per phase and re-syncing (Step 2).
- **Code changes only in the worktree; bookkeeping only on the default checkout.**
  Product/code edits go into `.worktrees/plan-<NNN>-dev`; `[✅]` markers and
  frontmatter flips stay on the default checkout's plan file.
- **Never modify a dependency project outside its own worktree**
  (`<dep-root>/.worktrees/auto-musk-dev`, named after this repo).
- **Every completed step gets a `[✅]` marker + `current_step` bump.** No silent progress.
- **TDD: failing test → implement → passing test**, when tests apply.
- **Blockers go to `## 待澄清事项`, not into speculative research.**
- **Follow steps exactly.** If a step looks wrong, stop and ask — do not redesign on the fly.
- **Do not start on the default branch without consent** (general safety rule).

## Checklist

- [ ] Target plan located; loaded as sole context
- [ ] Execution worktree `.worktrees/plan-<NNN>-dev` exists; every code edit/build/test ran inside it
- [ ] Dependency-project changes (if any) were made in that project's own worktree and folded back once consumed
- [ ] `status` advanced (`drafting → executing`, or already `executing`)
- [ ] Every execution step has `[✅ 已完成]` evidence
- [ ] `current_step` reflects the furthest completed step
- [ ] All per-step verification commands passed
- [ ] Blockers (if any) recorded under `## 待澄清事项`, not silently worked around
- [ ] On completion: `status: execution_done`; user pointed to `/auto-plan:review`
