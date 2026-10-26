# PLAN-853: diagnosis before R2 repair

Required checkpoint after two consecutive final needs_fix verdicts,760c232f4/R1 and5e3f5b65f/R2. This is a bounded completion-oracle repair under the unchanged r1 contract, not a semantic revision or a final-review pass. Sole tree D:/autostack/.wt/lang-853/auto-lang,plan-853-dev; repair base5e3f5b65f145fb127a4e206b626d3e2e3dc0ab8b. Both prior reviewers completed; no active writer. Implementation entry is clean. Shared main checkout unrelated code-path WIP remains untouched.

Before implementation, all three exact R2 raw inputs reproduced acceptance through BOTH check_selection and actual validate_run with the real unchanged measured PE; baseline-reproduction.json stores input hashes. R1 source/API closure and unaffected runtime gates remain reusable after confirming input identity; no performance/source/API campaign is restarted.

| Remaining invariant | Why prior detection missed it | Applicable states/consumer | Design/oracle adjustment |
|---|---|---|---|
| Every announced case completes exactly once in its own live block | R1 checked binary header/running/summary ownership, but counted only completed IDs. An open case was overwritten by a new case or discarded at summary; totals described the surviving subset. Unit tests mutated headers/results rather than in-block case transitions. | top-level/child pending -> new case; pending -> summary; zeroDoc -> any case; duplicate/unowned completion; check_selection -> validate_run | Track announced case identities separately from completed IDs; require no pending case before any new case or summary; enforce starts within declared count and completed==announced at exit; explicit owned completion only. Parent pending survives child block exit. |

## State transitions and closing evidence

- Header declares a binary; running/count creates its frame. Existing pending-binary checks stay.
- A frame has declared count, announced names, completed names, and at most one pending case. New start requires no pending case, a fresh identity, and room in the declared count. Immediate ok or explicit owned ok closes exactly that pending case. Failed/ignored never count as success; unowned completion rejects.
- A nested frame requires an open parent case; completing it restores that SAME open parent without auto-completing or overwriting it. The parent needs its own ok.
- Summary requires no pending case and exact announced/completed/declared equality, including zero-count frames. Binary EOF requires all blocks closed. Required-ID inclusion remains an additional check, never substitutes for complete observed state.

Permanent controls must exercise original three R2 files through selection AND validate_run, plus pending child/new case, parent missing completion after child, zero-count case, duplicate start and unowned completion. Accepted controls: simple/extra-complete binary, multi-case block, legitimate empty Doc-test, nested complete child including successive/nested children. Replay original76/280 logs each end and current scoped selections. Keep original R1 incomplete-header/unframed-result counterexamples rejecting. These checks distinguish a case-state invariant repair from fixing only one summary assertion.

No architecture/API/SDK change is needed. Work-owned closure remains pending until the state adjustment and controls pass on committed code. Review/merge-owned full gates stay pending; no automatic extra final-review cycle is relabeled as internal.
