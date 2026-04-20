# Plan: Create docs/plan-reports/ — Distilled Plan Reports by Topic

## Context

The `docs/plans/` directory has 194 plan files accumulated over 15 months. The design docs were just reorganized into topic chapters in `docs/design/`. Now we need a similar treatment for plans — but instead of moving them, we create distilled summary reports in a new `docs/plan-reports/` directory. The original `docs/plans/` stays untouched.

## Approach

Create one report file per topic area, summarizing all plans in that area with implementation status. Each report is a concise reference (not a copy of the plan contents).

## Directory Structure

```
docs/plan-reports/              ← new directory (docs/plans/ untouched)
  01-ast-core.md                ← plans 001-016
  02-type-system.md             ← plans 018-019, 021, 048-061
  03-error-handling.md          ← plans 008-010
  04-memory-ownership.md        ← plans 024, 034, 038, 088
  05-stdlib.md                  ← plans 020, 027, 041-043, 051-054, 102, 119, 143, 160
  06-transpilers.md             ← plans 007, 022-023, 062, 067, 083, 100, 152, 161-166, 170-175, 180-181, 187
  07-vm-runtime.md              ← plans 039, 068-081, 087, 117-118, 127, 177, 192, 194
  08-async-concurrency.md       ← plans 121-128
  09-modules.md                 ← plans 085, 089-091, 131, 167, 184
  10-build-tooling.md           ← plans 063-066, 092-093, 111-112, 146, 151, 186
  11-ui-generators.md           ← plans 094-099, 113-114, 133-136, 138, 142-147, 174-175, 180-181
  12-testing.md                 ← plans 110, 158, 170-172, 179, 191
  13-self-hosting.md            ← plans 028-033, 037, 095
  14-language-features.md       ← plans 035-036, 040, 044-045, 050, 082, 084, 086, 139, 155-156, 168-169, 182, 185, 190, 193
  15-documentation.md           ← plans 032, 097, 103-109, 132, 137, 141, 144-145, 148-150, 157, 183, 188-189
  16-shell-tools.md             ← plans 017, 046-047, 153, 159
```

## Report Template

Each report follows this structure:

```markdown
# NN - Topic Title

## Overview
[2-3 sentences describing what this area covers]

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 001 | VM Function Integration | ✅ Done | One-line description |
| 002 | to-atom AST | ✅ Done | One-line description |
| ... |

## Status Summary
- Completed: N plans
- Partial: N plans
- Planned: N plans
- Deprecated: N plans

## Key Achievements
[2-3 bullet points on what was accomplished]

## Remaining Work
[2-3 bullet points on what's still planned]
```

## Implementation Steps

1. Create `docs/plan-reports/` directory
2. For each report (16 total):
   a. Read the relevant plan files from `docs/plans/`
   b. Extract title, status, and key points from each plan
   c. Write the report following the template
3. Commit (docs/plans/ remains untouched)

## Verification

1. `ls docs/plan-reports/` — 16 report files
2. Each report has the table format with status
3. `ls docs/plans/` — original 194 files unchanged
