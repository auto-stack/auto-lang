---
name: auto-plan-review
description: |
  Review a finished plan against its acceptance criteria and the actual code,
  then fill in the spec-impact metadata (supersedes / new_spec_components /
  touched_goals) so /auto-plan:merge knows what to deposit. Sets status to
  reviewed (pass) or sends back to /auto-plan:work (fail). Use when:
  (1) User says "review plan N" / "复审 plan N" / "验收 plan" / "check plan N"
  (2) User says "/auto-plan:review" or a plan just reached execution_done
  (3) User wants to verify a plan is truly complete and ready to merge into the spec ledger
  This skill never merges — it only verifies and prepares metadata. Verification
  is re-run from scratch; a green checkbox in the plan is a claim, not evidence.
---

# /auto-plan:review — Review a plan for merge-readiness

Re-verify a plan that is *believed* done, fill in the spec-impact metadata that
`/auto-plan:merge` consumes, and route the plan to `reviewed` (pass) or back
to `/auto-plan:work` (fail). One skill, one session, one review pass.

> **Design source:** `docs/designs/008-auto-plan.md` §6.4. This skill is the
> plan-specific review gate; for the generic "close out + route to archive/debt"
> pass, the broader `/finish-plan` skill applies afterwards.

**Announce at start:** "I'm using /auto-plan:review to verify plan `<NNN>`."

**Input:** A plan reference (number / filename). The plan should be
`execution_done`; if it is `executing` with all steps marked done, this skill
will advance it to `execution_done` as the first action.

**State gate:** `execution_done` is the expected entry state. `reviewed` or
`archived` means review already happened — re-review only if the user explicitly
asks. `drafting`/`executing` with unfinished steps → refuse and point to
`/auto-plan:work`.

## Process

### Step 1: Load the plan AND the actual code

Read the plan file (it lives on the default checkout). Then look at the real
diff and the real files it claims to have touched — from inside the execution
worktree `.worktrees/plan-<NNN>-dev`. If the worktree is already gone, its
branch was folded into main early; verify against the default checkout instead.
Plans drift from implementation — **trust the code** when they disagree.

```bash
git log --oneline -20                 # what actually committed
git diff <branch-base>..HEAD --stat   # what actually changed (run in .worktrees/plan-<NNN>-dev)
```

### Step 2: Re-verify every acceptance criterion

The plan's `## 验收标准` section lists checkboxes. For each one, reproduce the
verification yourself — do not trust a checked box:

| Criterion type | How to re-verify |
|:---|:---|
| Test suite passes | Run the exact command (`cargo test ...`, `npm run build`, etc.) |
| API endpoint works | `curl` it, or read the handler + its tests |
| File/feature exists | Open the file; confirm the claimed behavior |
| Type-check / lint clean | Run `vue-tsc`/`cargo check` and look for new errors |

Record each as pass / partial / fail with a `file:line` or command-output
evidence note. **Any unfinished or workaround-forced item is recorded, not
hidden** (it becomes a debt candidate).

### Step 3: Hunt for 遗漏 / 延后 / workaround (the lazy-convergence check)

The executor optimizes for finishing fast. To converge quickly it tends to
silently drop part of a task, defer it "for later" once it finds any excuse,
or paper over it with a workaround — then report the whole plan complete.
Assume this may have happened; hunt for all three patterns explicitly:

- **遗漏 (dropped):** a task marked Done that lost a sub-item (a test, a
  call-site update, a config)? A plan-level task with no corresponding change
  in the diff at all?
- **延后 (deferred):** anything postponed to "a follow-up plan / later batch"
  without the user approving the split?
- **Workaround:** any `// TODO`, hack, scope reduction, or "works but not
  clean" approach — especially one forced by an upstream limit?

Each finding is a debt candidate: record it with the root cause. A deferral
the user never signed off on means the plan is *not* actually complete — fail
the review and put it on the fix list; recording it as debt alone does not
make the plan pass.

### Step 4: Fill in the spec-impact metadata (the key step for merge)

Analyze what this plan actually changed in the spec ledger's domain, then
populate the frontmatter fields `/auto-plan:merge` will read:

```yaml
supersedes_spec_components:   # existing spec items/modules this plan modified
  - "specs/modules/<x>/spec.md: 修改"
new_spec_components:           # new spec items/modules this plan introduced
  - "specs/modules/<y>/spec.md: 新增"
touched_goals:                # which goals this plan advances
  - "goal-001: <one-line>"
```

These must be **precise** — `/auto-plan:merge` uses them verbatim to decide what
to upsert. If a field does not apply, leave the list empty rather than guessing.

### Step 5: Write the review record + route

Fill `## 复审记录` with: reviewer, time, per-criterion verdicts, any debt
candidates. Then route:

- **All criteria pass, no blocking debt** → `status: reviewed`. Tell the user
  the plan is ready for `/auto-plan:merge`.
- **Any criterion fails or the plan is not actually complete** → keep status at
  `execution_done` (or roll back to `executing`), list exactly what to fix, and
  hand back to `/auto-plan:work`.

## Rules

- **Verify, don't trust.** A checked box is a claim. Re-run every verification.
- **Re-run verifications inside the execution worktree**
  (`.worktrees/plan-<NNN>-dev`); write 复审记录 and status flips to the plan
  file on the default checkout.
- **Never fold the branch back here.** Landing `.worktrees/plan-<NNN>-dev`
  onto main happens in `/auto-plan:merge`, after this gate passes.
- **Trust code over plan text.** Record divergences in the review record.
- **Never set `reviewed` on unverified work.** Partial → fail the review.
- **No silent deferrals.** Postponing a task, shrinking scope, or swapping in
  a workaround without saying so counts as incomplete, not done.
- **Metadata must be precise.** `/auto-plan:merge` reads it verbatim.
- **Never merge.** This skill stops at `reviewed`; merging is `/auto-plan:merge`.
- **Defer to specialists.** After `reviewed`, the broader `/finish-plan` can
  run for the generic close-out + archive routing.

## Checklist

- [ ] Plan loaded alongside the actual code diff
- [ ] Verification re-ran inside `.worktrees/plan-<NNN>-dev` (or on the default checkout if already folded)
- [ ] Every acceptance criterion re-verified (pass/partial/fail + evidence)
- [ ] 遗漏 / 延后 / workarounds hunted explicitly and recorded
- [ ] `supersedes_spec_components` / `new_spec_components` / `touched_goals`
      filled precisely (or left empty if not applicable)
- [ ] `## 复审记录` written with per-criterion verdicts
- [ ] Status routed: `reviewed` (pass) or back to work with a fix list
