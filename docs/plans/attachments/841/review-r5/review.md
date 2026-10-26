# PLAN-841 r1 — R5 affected final review

2026-10-27; outcome: **needs_fix**; review_scope: final (affected). This Codex /root session implemented the submitted repair in the preceding turn and reconstructed this review from committed artifacts; it is not a fresh independent context. No agent was created. Implementation was frozen; temporary targeted probes were removed afterwards. No implementation change, canonical publication, ledger edit, whole-tier gate, merge or cleanup occurred in this review.

## Baseline and scope

Shared Plan read from the language main checkout at execution_done 9/12. Existing group D:/autostack/.wt/lang-841, branches codex/plan-841-dev: lang 88cd1aa7bdd4ac31d16b8d9dc229c5bb30bcbe80, UI 2f311868f082bc4ab6fac9d484f71f18d13b1c95, Man 9b4302fa585e6cda4a91738a6e9057fae12c9402. Down remains fixed c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 (read-only clean clone used for delivery; foreign dirty down tree untouched). Three source trees clean on entry and exit, heads unchanged. [Frozen inputs/config/probes](baseline.json.txt); exact shared Plan input is base64-preserved. The work baseline records language before its docs-only repair descendant; HEAD diff confirms the descendant adds only candidate Spec/Plan/evidence.

Review starts from [M841-GATE-04](../merge-r1/M841-GATE-04/finding.md) and [work readiness](../repair-g04/closeout.md). Inspected queue acquisition, duplicate/replacement, replay/root ownership, missing/cancel/drop, typed array storage and scalar distinction, actual property producer, graph canary assertions and candidate SD-03. Rustfmt and repair diff whitespace checks pass. Man change is only the actual UI source pin and matches reviewed UI HEAD; product code/locks/features unchanged.

## Results and findings

M841-GATE-04 original example is fixed: the literal undefined-name input still rejects before VM construction; same-file repair recovers; parse-valid integer zero division reaches BarChart Init Failed/DivisionByZero, Line/Area/Donut remain Done. Actual hit counts healthy36/runtime14 retain the original relative comparison and >=12 sibling floor. No parser relaxation, skipped test, swallowed module error, disabled RC canary or wider timeout. Direct-reference state transfer and tested terminal paths are improved.

The shared invariant is **not closed**. New **F841-R5-01 P2 OPEN**: already-accepted nested-array and object-in-array prop values retain a borrowed VM reference without queue/root ownership. Replacing the original parent field frees the object while Child Init is still Queued. Both probes assert the actual producer's shape and exact reference id before testing liveness; a direct-array control passes. See [full finding, literal inputs and closing oracle](finding-F841-R5-01.md).

| Check -> invariant | Actual result | Evidence |
|---|---|---|
| Existing M1 canary/healthy + five new permanent snapshot controls + R1 invalid module controls | **9 pass** | [permanent controls](permanent-controls.log.txt); actual graph counts and failure/recovery included |
| Shared owner across demand cancellation; aliased fields/same-value reseed; CPU-parked Init identity replacement | **3 pass** | [additional owner cases](additional-owner-controls.log.txt), [probe](review841_g04.rs); forced collection and last-owner release |
| Actual direct/nested-array/object prop producer -> queue after parent overwrite | **1 pass / 2 fail** | [compound producer cases](compound-producer-controls.log.txt), [probe](review841_nested.rs). Initial two-case run retained but not counted again |
| Format / diff / source pin / input hashes | pass | committed repair checked; no implementation drift |

Final newly executed scope: **15 distinct cases: 13 pass, 2 fail**, no selected-case skips. Earlier work's same-harness M4, state-neighbor and six lifecycle results remain valid for their unchanged inputs and are not rerun or included in the review total. Raw aggregate successes do not override the two actual accepted-input lifetime failures.

## AC and Spec disposition

- AC-01/02: unaffected classification/Core independence, admission and syntax evidence retained from R4 and earlier reviews. Compound cases are themselves accepted through the real domain parser/lowering.
- AC-03: prior codec/deep-UI/source/Weak evidence retained on unchanged code; **runtime snapshot ownership partial/fail** for supported compound props.
- AC-04/05: original package canary/direct owner paths pass, but accepted compound consumers and full failure/lifetime coverage **fail** (F841-R5-01).
- AC-06: needs_fix; acceptance and candidate rule are not ready for final pass.
- AC-07: pending, merge-owned.

Candidate-only [reviewed SD repair delta](spec-delta-reviewed.diff), SHA256 14cc305bfe2db4ca0b0aed6c6d892f5ae1e6e11296fada9a46a65aefd4950718. Target docs/specs/auto-ui-runtime/project.md is correct; SD-03 is the existing component, no new capability or Plan revision is invented. However, the broad snapshot-ownership rule is not implemented for accepted compound values; the reference to direct ListData elements is not approval to narrow producer support. Other unchanged SD-01..05 evidence remains scoped to its prior baseline.

## Route

Plan returns **executing 6/12**; reopen T-07/T-08/T-09 and leave T-10 pending. Preserve T-01..06 and all historical failures/pass evidence. Original G04 old-input closure is distinguished from the still-open shared invariant and new repair coverage gap. Next owner work: fix F841-R5-01 with the concrete producer/control matrix and queue plus replayed-container reclamation, commit and submit affected review. No aggregate rerun on known incomplete acceptance. Bare t, applicable tv/tt/tu, UI/Man tiers and frozen 777/792/distribution remain merge-owned pending; old gate results do not certify the new repair. Three implementation trees retained clean and no U841 completion is claimed.
