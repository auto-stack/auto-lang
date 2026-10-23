# PLAN-837 F-837-R1-01 repair readiness

stage: work | outcome: pass | execution_done8/10 | next: affected independent re-review.
Code2ffc92d34143bc97a98f2684406160b183fd2ff3, plan-837-dev, same authoritative
D:/autostack/.wt/lang-837/auto-lang. Reviewed baselineabe682c5a, Plan revision1.

| Finding | Invariant and affected states | Fix / permanent cases | Current result / remaining gap |
|---|---|---|---|
| F-837-R1-01/P2, AC05/06/08, T07/08 | Actual visible endpoint must match initial leaf color, old position must match background, hidden both background; temporary/move/Undo/Redo/Show/saved-unselected; missing readback fails | Probe::pixels reads final top RGBA and stores actual/expected before asserting. Aggregate requires all7 state records. Permanent tools/check_hierarchy_pixel_faults.py renders transparent endpoint or extra old-position ghost, preserves valid state/commands, then demands same native/report/aggregate rejection | Normal phase16/204frames,7 passed pixel records; old parent/release/partial negatives retained; new-process saved v2 document/world/2D+3D capture/clean-history-selection passes. Actual transparent and ghost each native phase9/exit1/passed=false, aggregate1 at composed endpoint/origin mismatch. No work gap; independent re-review pending |

Original review replay SHA25660727913a42181040dd0b9041d6799899930386e4a1a7efb38b1f9757dddbd7a copied byte-exact and rerun before/after, preserving review artifacts. Pre-fix actual transparent GPU still passed native+aggregate phase16; post-fix original replay is rejected at phase9 actual-pixel assertion with native/report/aggregate failure. Both fresh outputs remain under pre-fix/ and post-fix-review/. The permanent second ghost control independently proves vacated-origin sensitivity while endpoint remains correct. This is actual GPU content, not counters, commands or paired wrong hashes.

3 targeted completion/CLI tests pass,0 ignored; all-targets check and fmt pass.
Source247/SDK680/479packages/17locals/29edges plus drift/missing-fixture/GLB controls pass. Fresh external source consumer independently passes source/controls/all-targets check. SDK146 inherited warnings unchanged, no authored warning. No full driver/AGP/root tier ran; merge owns the full frozen matrix after final review.

No authoring library/tests/AGP/SDK/locks/game/assets changed. R1's38-test graph/Auto/codec/transaction/matrix/numeric/resource results are reused with fingerprints in receipt.json.txt; old visible-pixel claim is invalidated, not reused. No general renderer workaround, support reduction, new product decision or canonical/ledger publication. README, handoff, feasibility, proposed SD05 and source identity refreshed. Exact commands and scope are in tree research/hierarchy-repair-r1.md.

Final review count remains1 failed R1; this implementation-context pass is not an independent review. T07/T08 work-side completed, T09/T10 pending. Same tree retained, clean after commit; implementation paused for independent re-review. No merge, archive or cleanup is claimed.
