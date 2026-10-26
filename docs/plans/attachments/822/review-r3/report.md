# PLAN-822 final repair review R3 — needs_fix

Date: 2026-10-26. Plan revision 5. Reviewer: current chat Codex; did not implement the repair; independently inspected the committed diff and ran the recorded probes. No delegated reviewer or fabricated model identity.

Baseline: worktree D:/autostack/.wt/lang-822/auto-lang, branch plan-822-dev, HEAD bf9e92cb472495037ce73daab5b0c1d2a4f09f7d. Repair base 81c33e0918468bc21f6bd756fb8bbd6b4616ab57; complete Plan diff base de93bacc0ad16622140b0adb4d36eee86fe7fc83. Cargo.lock SHA256 37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873. Canonical inputs / contract / SD hashes in input-hashes.txt; exact proposed delta preserved as sd-frozen.md. No canonical Spec or ledger writes.

At entry, the implementation tree already contained untracked probe/tests/review_822_r3.rs. This review did not author it or attribute it to the executor. Its exact bytes were preserved as replay.rs; all four inputs and assertions were independently inspected and replayed. It remains untouched in the tree. All tracked implementation files were clean before and after. Probe-only unused import/macro/helper warnings are from this pre-existing untracked file; the committed targeted tests below compile without warnings. These are not claimed as new production warnings.

## Prior closure and targeted checks

- R2-01: reported failed-candidate publication defect closed on the examined link/trap paths: apply_revision builds candidates, links and executes before assigning current units/identity/revision. Two immediate-current-state tests pass (session.log.txt).
- R2-02: Place mint/epoch/generation mechanism now exists through MIR/ABC/AV. Five existing legal/stale/out-of-range controls pass (place.log.txt). Full closure withheld: new path does not preserve typed elements (R3-01).
- R2-03: original forward-order example closed; three fixed event-oracle tests for normal/trap/nested variant exit pass (cleanup.log.txt). Full lifecycle closure withheld: shared transfer cleanup now destroys a moved list (R3-02).
- R2-04: original skip/missing/zero/duplicate controls now reject and valid control passes (guard.log.txt). Lists explicitly bind the new mandatory semantics/incremental names; no aggregate acceptance replay was run. This closes the specific reported validator gap, not every remaining AC/V requirement.
- R1-01 prior budget-order evidence retained; no unrelated rerun. R1-02's whole AC-09 is still partial: session repair evidence is valid but this round does not replay unchanged scalar campaign/performance or approve every dependency axis.

## F-822-R3-01 — P1: new Place access bypasses typed collection admission and execution

Existing rule: Plan §5.1.1/5, AC-01/02/04, V-01 require exact element types and malformed typed fixture rejection before consumption; AC-03 also requires owner/type integrity. R2 Place delivery cannot weaken these rules.

Locations: core/src/mir_verify.rs:2164–2213, core/src/av.rs:2643–2757. PLACE_GET accepts any non-owned destination; PLACE_SET accepts any value type. MirType::Place carries no referent type and the runtime neither compares source/destination element type nor recovers/checks the root's element type. The comment promising runtime validation is unsupported by either path.

Reproduce (worktree cwd): `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r3 -- --nocapture` (if reconstructing, copy replay.rs into probe/tests/review_822_r3.rs). Legal base: List<i32> with one element 7, index 0, LIST_PLACE. `review_r3_place_get_wrong_type` uses a bool destination; expected controlled admission refusal, actual verify_mir=Ok, ABC admission and execution succeed. `review_r3_place_set_wrong_type` writes true through that place then LIST_GET into the i32 return slot; expected refusal before mutation, actual verified/ABC-admitted execution Success { value: Bool(true) }. That concrete wrong return value demonstrates actual list corruption rather than only a missing static annotation. Both assertions fail. `review_r3_legal_place` is the accepted unchanged-type control and returns I32(7).

Cause established from full path, not hypothesis: verifier has no referent-type constraint; resolver validates identity/epoch/index only; PlaceSet unconditionally replaces the element, PlaceGet commits it to the chosen slot. No runtime type backstop exists.

Affected siblings / closing oracle: typed read and write, bool/i32 and owned/nominal element identity, Place transport/redefinition and supported control-flow paths. Bind/check element identity across the declared supported paths (or reject unsupported shapes explicitly), then replay invalid read/write with expected stage/code and zero target access/commit; legal read/write and current stale-token controls must remain valid. Do not merely turn the corrupted result into a later unrelated trap or silently drop Place support. Scope is exact typing in this profile, not new generic/source support.

## F-822-R3-02 — P1: reverse-cleanup repair empties transferred list

Existing rule: Plan §1/§5.1.2/6 and AC-02/03 require list move/ownership transfer to preserve the destination value and cleanup each owned responsibility once.

Locations: core/src/av.rs:859–897 and 1552–1565. Move clones the ListHandle into the destination, then calls owned_kill_slot on the source. The new owned_kill_slot calls drop_owned_reverse on that handle; drop_owned_reverse unconditionally takes/drains the shared buffer's elems. Source and destination share the same buffer, so destroying source responsibility destroys destination contents. This is introduced by bf9e92cb4's cleanup change (previous owned_kill_slot only released the source handle).

Same replay command, case `review_r3_move_preserves_list`: after constructing List<i32>[7], MOVE to another List<i32> slot and LIST_LEN on destination. Expected I32(1); actual verified/ABC-admitted Success I32(0), assertion fails. There is no out-of-domain nesting, stale token use or resource fault. The legal base Place control also proves the initial element exists. Logs in replay.log.txt (overall exit 101, 1 passed / 3 failed / 0 ignored).

Affected shared callers: owned_kill_slot is also used by owned call-argument transfer, container/variant source consumption, field transfer and explicit Drop. Distinguish moving a responsibility from final destruction; inspect those supported callers rather than patching only the len example. Closure needs non-empty list move result/content and owned-element lifetime controls (e.g. List<Str>/record), supported call transfer/return paths and normal/trap reverse-cleanup controls. Keep existing resource precheck guarantees. No Plan-wide regression is requested for each finding.

## Acceptance / knowledge / next action

| AC | This round |
|---|---|
| AC-01 | partial/fail for typed gate integrity (R3-01); original first-case/oracle evidence retained, not globally rerun |
| AC-02/03 | fail: typed Place corruption and moved-list data loss; valid existing Place and cleanup controls do not cover these siblings |
| AC-04 | fail: malformed typed Place fixture admitted and consumed |
| AC-05 | partial: old transaction evidence retained; lifecycle invariants not fully closed; unchanged memory campaign not rerun |
| AC-06 | partial: previous cache/compatibility/fresh-worker audit boundaries retained; not newly certified by unrelated tests |
| AC-09 | partial: failed-candidate paths now pass; unchanged source/performance evidence not replayed; no whole-invariant pass inferred |
| AC-07 | fail: findings remain |
| AC-08 | pending merge, not authorized by this review |

V-01/02/06/11/12 affected by findings; V-13 link/trap transaction subset repaired. V-03/07/08/09/10 unaffected prior evidence is retained at its previous confidence/boundary. Unexamined remaining criteria stay pending; no green total substitutes for full contract verification. Because code is known to need repair, no global suite/performance matrix/aggregate acceptance was spent to reconfirm that conclusion.

SD-01/02 cannot yet deposit typed Place/lifecycle claims. SD-06's build-aside statement is supported for the inspected link/trap paths; no full SD/AC pass. Existing exact metadata targets and empty touched_goals rationale remain valid; before/after summaries still require final verified content at closeout. Hashes/frozen delta bind this verdict only.

Next: status executing; reopen T-03/T-04/T-06, preserve T-01/T-02/T-05/T-09 and historical records, current_step 4/9. Work in the same tree must provide diagnosis/coverage adjustment for shared typed Place and transfer/destructor invariants, targeted oracle results and committed repair before re-review. This is the third failed final review: do not submit only old-example totals as proof of closure. No implementation fix, merge, archive or cleanup performed.

Merge-owned gates remain pending: after final code/acceptance/Spec pass, freeze commit and run bare cargo t + applicable cargo tv + experimental workspace finite contract checks and any actual additional surface tier. cargo tf stays batch-only on main. No whole-tier cargo t/tv/tt/tb/tu/taa or aggregate acceptance ran in R3.
