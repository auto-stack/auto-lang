# PLAN-771 independent final review R3

2026-10-14, plan_revision 2, review_scope final. Reviewed commit e0ad328d80bfe00ca7563174b3352edeb8e6347b, base 199d6509e60821c3c0a6414ecd0bedf484105e4d, prior review 86f3a91a0cabfafede9b9b69579d368bdf3df522. Worktree D:/autostack/.wt/lang-771/auto-lang, branch plan-771-dev. Current Codex review context did not implement either repair. No separate agent/model identity claimed. Implementation tree clean and HEAD stable before/after independent tests; no implementation edits. Main observed HEAD 047c2696306afbe65ddbf0a712d50cd3d3b16681.

## Diagnosis and invariant audit

Work supplied the required two-failed-review diagnosis, explaining the distinction between destination overlap (R1) and deferred source value capture (R2), and changing coverage accordingly. ReadLocal ctor value operands now materialize at their own evaluation point (native.rs record_eager_reads:307, Enter ReadLocal:1304). Dest storage reuse remains gated by whole-expression record mention sets. Record-valued FieldGet materializes during field evaluation, record-returning calls have their own out storage, and nested constructors complete before later sibling effects. ReadLocal under BorrowView retains a pointer; BorrowMut remains a pointer. The repair does not globally copy borrowed parameters or suppress mut conflict diagnostics.

Review inspected complete insert/consume paths for dests/elided, fresh local and ABI-E out storage, whole-local and nested field assignment, call results and constructor field binding. Eager ReadLocal plus CtorOut avoids delayed source copying; the first repair's disjointness proof avoids premature target overwrites. Iterative graph/parse/resolve/Drop/layout and 1024-type/4096-path evidence remain within the existing declared validation scale, not a language depth ceiling.

## Frozen old counterexamples replayed unchanged

| Input in prior review attachments | R3 result |
|---|---|
| R1 alias-ctor.at / alias-sret.at / alias-control.at | each build success, PE exit0 |
| R1 positional.at | build success, PE exit0 |
| R1 actual Bundle mutation harness | valid control verifies; binding-arg and expression-ref both controlled rejection, no panic |
| R2 ctor-snapshot.at / positional-snapshot.at / snapshot-control.at | each build success, PE exit0 |

Input identities remain those in R1/R2 manifests. Rebuilt harness linked against final ac-core rlib using the original rustc command with Windows native dependency library path; no implementation test files added by reviewer.

## Independent combinations and oracle sensitivity

New combined-snapshot.at tests a function returning a constructor using an explicit mut parameter, a mixed i32/bool record, record-valued field reads, and three repeated loop iterations with later source mutation. Build success and exact PE exit0 (checks capture-before-mutation, mutated source, bool preservation and iteration-dependent values). Initial reviewer draft lacked parentheses around arithmetic comparison RHS and was rejected at parse; the saved corrected input uses `(9+n)` and `(10+n)`, with no language/runtime failure inferred from the discarded syntax draft.

Direct HIR r04 has both read-before-mut (snapshot=old value) and mut-before-read (snapshot=new value) assertions, and distinct success0/failure31..34. Independent mutation of c1b expectation to99 forces PE exit32; independent mutation of c6 to99 forces PE exit34. Saved hir-snapshot-failure-control.atom and hir-reorder-failure-control.atom prove the actual PE assertions distinguish both directions. Unmodified r04 exits0 in tests and gate. Existing r02 intentionally failing copy fixture exits11 and cannot satisfy success0.

Permanent new tests reviewed: seven source variants (named, positional, fresh control, mut-first reorder, nested ctor, partial mutation, returned operand), direct HIR order test, and negative `mut_exclusivity_spans_ctor_arg_trees` controls that require verify.mut-conflict for view+mut across combined call argument trees, including constructor operands. Existing aliases, positional diagnostics, name shadowing, identity, nominal types, graph cycles, signature/receiver references, layout/call/sret, trace ba/aab, receipt and SENTINEL tests retained.

## Acceptance mapping

| AC | Tasks / implementation | Independent evidence |
|---|---|---|
| AC01 | T02/03/06, descriptor/HIR/writer/profile receipt | record_hir identity and isolated profile gates |
| AC02 | T03/04/05, constructors/field/assignment/native | R1/R2 source PE replay, HIR r01/r03/r04, new mixed and loop probe |
| AC03 | T02, verifier/type graph/signature/receiver | actual Bundle replay and full record_hir family incl bounds/cycles/mutations |
| AC04 | T04/05, aggregate params/results/methods/mut | ABI PE family, alias controls, mutable-param returning ctor, exclusivity negatives |
| AC05 | T03/05, evaluation order/span/deep/Drop | ba/aab traces, bidirectional r04 controls, 1024/4096 native and malformed2000 regression |
| AC06 | T06/07, gate/SD/inventory/product identity | current gate, old profile gates, scope audit and frozen SD/final product fingerprints |
| AC07 | T08, independent audit | complete review with original and broader invariant closure |
| AC08 | T09 | pending merge-owned closeout |

## Findings closure and knowledge delta

F-771-R1-01..04 and F-771-R2-01 now closed for the agreed bounded profile. No new blocking finding or unapproved scope reduction identified. SD01..05 reviewed against unchanged current canonical inputs (spec-inputs.sha256 matches R2): positional and named finite record surface, trusted HIR and iterative graph checks, value/borrow/order/storage obligations, experimental native ABI limitation and bounded ACC attribution consistent with implementation. Frozen proposed-spec-delta.frozen.md is the reviewed deposit proposal, not published canonical text. Its source-order language encompasses eager capture; merge should explain that lasting rule rather than an implementation diary. Exact metadata target paths remain valid. touched_goals=[] because no canonical goal including GOAL017 is closed by this experimental subset.

Nonblocking F-771-R3-01: verification.md §4 still labels its current table full tests187/187 and native9/9 with old timings, despite updated overview202/202 and native15/15; §4c claims those rows were removed. Current review's durable receipts resolve actual results. Fix this documentation inconsistency during closeout; it does not invalidate executed criteria or require weakening the contract. No implementation repair needed for this note.

## Replay and closeout boundary

All replay build-source commands use profile auto.source.record.draft / entry main; HIR uses build / entry d_main, under experimental/ac-core/target/debug/auto-ac-prototype.exe. Execute unique output PEs and compare the exact codes above. Gate invoked from the same worktree: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-record-771.ps1. Durable receipts/hashes attached; context records entry binary, final-products.sha256 records final rebuilt binary and support library actually used.

772/774 remain archived and dependency code unchanged; 770 is active, not merged. Any later implementation/integration baseline change invalidates affected evidence. If 770 lands first, 771 must replay its full gate plus 770 gate on the new combined baseline and receive fresh review before merge as Plan §8/interface-contract §6 require. Actual merge, canonical deposit/ledger/index/archive/counter/guard cleanup remain pending. Review performed none of those operations.

## Final verdict

PASS (final), bound to plan_revision2 / e0ad328d80bfe00ca7563174b3352edeb8e6347b. Independent gate24/24 PASS, 23 cases, full ac-core202/202, record_native15/15, fmt/all-targets zero warnings, old741/745/767 PASS. AC01..07 pass; AC08 pending. Plan reviewed8/9, T08 checked, T09 open. F-R3-01 is documentation hygiene only. No implementation/dependency/config drift at final recheck. Next: merge workflow subject to existing authorization and fresh-baseline conditions above.
