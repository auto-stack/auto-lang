# PLAN-822 final repair review R5 — needs_fix

2026-10-26; Plan revision 5. Reviewer: current chat Codex, independent of implementation and repair; no delegated agent. Review scope: final, affected repair/invariant audit, not a merge gate.

## Baseline and evidence identity

- Existing implementation tree: `D:/autostack/.wt/lang-822/auto-lang`, branch `plan-822-dev`.
- Reviewed commit: `ad9a69ffa12919ac8812b18b78a0b481b6f319bb`; repair base: `7239e8f4774bc4f6ea7cc6354bd78f99f780b55a`; full implementation base: `de93bacc0ad16622140b0adb4d36eee86fe7fc83`.
- Main bookkeeping HEAD: `4ddb00b3e26d6f2addc62f04f05604ad258fb3f7`. Shared Plan read from main. D821 commits `01747f278`/`5009dc629` are ancestors of the implementation. Existing contract dependency/window receipts retained.
- Experimental Cargo.lock SHA256: `37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873`, unchanged. Canonical Spec, contract-decision and SD hashes match R4, recorded in `input-hashes.txt`. Plan hash there is before R5 bookkeeping.
- Frozen proposed SD: `sd-frozen.md`, SHA256 `4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739`. No canonical Spec or ledger writes.
- Entry/final HEAD identical, tracked implementation clean. Pre-existing untracked `review_822_r3.rs` retained untouched (SHA256 `3F84BA196CFDDF2853D5102D38FEA83FF45DBC4E09955A65738A0F10988306D8`). This review's temporary `review_822_r5.rs` was saved as `replay.rs`, then removed. Product code not repaired.

## R4 closure and checks

The work diagnosis matches the repair: full type-table validation omitted Place.elem while downstream snapshot remapping assumed a valid referent. The repair validates every Place entry, including unused ones, for bounds and the same element domain as List.

- `replay-r4.log.txt`: original independent R4 replay, **1 passed**. Invalid `Place<u32::MAX>` is rejected by both MIR and real ABC admission as `mir.type-ref`; actual CollectionsUpdateSession returns Err without panic. Legal Place<i32> uses the same path, returns 7, live=0, allocs=frees=1. **F-822-R4-01 CLOSED** on this baseline.
- `repair.log.txt`: committed `place_referent` tests, **2 passed**, no ignored. Includes real failed-candidate session preservation, legal control, boundary, self-reference and unsupported container referent.
- `place.log.txt`: existing semantics `place_` tests, **8 passed**, no ignored. Valid local reads, typed mint/get/set rejection, stale epoch/fresh index, element generation and bounds controls remain green.
- `new-replay.log.txt`: independent related-path probes, **2 passed / 3 failed**. Passing controls are cross-frame read=7 and entry-result profile rejection. Three failures establish two findings below. The entry-result rejection is not claimed as a borrow-provenance test; the nested callee return exposes the missing check.
- `git diff --check 7239e8f47 HEAD` clean; targeted tests compiled without warnings. No whole-tier, full workspace, aggregate acceptance, memory campaign or performance replay was run after a contract blocker was established.

Replay setup: copy `replay.rs` into the existing implementation tree as `experimental/ac-av-probe/probe/tests/review_822_r5.rs`, then run:

```text
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r5 review_r4 -- --nocapture
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r5 review_r5 -- --nocapture
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_collections_822_incremental place_referent -- --nocapture
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_collections_822_semantics place_ -- --nocapture
```

The initial probe used nonexistent CalleeRef::Function; corrected to actual CalleeRef::Func before behavioral evidence. An initial top-level return probe was a unit, so its execute result was non-executable-unit Infra; corrected to a whole module. Whole-module entry-result rejection is an accepted control, not an escape counterexample. The final nested-callee probe has an i32 entry and reaches actual execution; only its actual MIR/ABC acceptance and InvalidHandle result underpin F-R5-02.

## F-822-R5-01 — P1: PlaceSet validates the owner frame but commits in the current frame

Affected: AC-02/03/04, T-03/T-04; V-02/11 and the shared Place ownership invariant. Original contract gap in the R2 Place implementation, newly exposed by related-path review; not introduced by this 18-line repair.

Existing rule: §5.1.3 requires typed Index Place access with identity/epoch checks before access, §5.1.6 requires controlled lifecycle/failure behavior, AC-04 prohibits panic as a negative outcome. An admitted program cannot use a successful validation of one object to access another object unchecked. This finding does not require adding a new cross-function language feature: an unsupported shape can be explicitly refused before execution.

Bound input: `cross_frame_place(true)` / `review_r5_cross_frame_place_write_no_panic` in replay.rs. Caller creates List<i32>[7] in slot 0, mints Place<i32> in slot 4, passes it to helper(param Place in helper slot 0). Helper materializes i32 9 in slot 1, then PLACE_SET(param, 9). Matching types, no forged credential or invalid index; caller remains active.

Expected: accepted support writes the referenced caller element (subsequent caller read=9), or an explicitly unsupported shape is rejected at admission; no panic or unrelated-slot write. Observed: both real MIR/ABC gates accept; execute panics at `core/src/av.rs:2735`, `unreachable!("validated above")`. catch_unwind is only in the independent test, not the production executor. The sibling helper PLACE_GET uses the same caller token and succeeds, result=7 (`review_r5_cross_frame_place_read_control`).

Established cause: `validate_place_for_write` around av.rs:992 locates the token's owner via frame sequence and validates its slot/buffer. PLACE_SET commit at av.rs:2726 instead selects `st.frames.last_mut()` and indexes `place.slot` in the helper. Its slot 0 holds Place, not List, triggering unreachable. If another frame's same-numbered slot holds a list, the same code would target that unrelated buffer; that second shape is a source-derived risk, not an independently executed result.

Closure: reconcile admission/support and frame resolution for the entire read/precheck/commit path. Replay the panic input, retained local get/set and stale controls; if cross-frame access remains accepted, assert caller content becomes 9, unrelated callee data unchanged and live=0. Check a representative differing slot arrangement and invalid/dead-owner failure before target mutation. Do not add general regression/performance campaigns to this finding.

## F-822-R5-02 — P1: Place bypasses borrow-escape and record-field restrictions

Affected: AC-02/03/04, T-03/T-04/T-06; V-01/02/11, SD-01/02. Original Place integration gap, not a new feature request.

Existing rule: §5.1.6 explicitly requires borrow escape rejection; contract-decision §2 preserves 821's record-field domain and rejects borrowed fields/tokens. Place is documented in mir.rs as a retained element reference, carrying frame/slot provenance rather than owning its root. A runtime invalid-handle check does not replace the required static rejection of a provably local escaping borrow.

Bound inputs and actual results:

1. `review_r5_callee_local_place_escape_refused`: whole module with i32 entry calls helper() -> Place<i32>. Helper creates its own List<i32>[7], mints a Place into it and returns that token. Entry performs PlaceGet on the result. MIR and ABC admission both succeed; execution reaches the caller and reports `Trap { kind: InvalidHandle }`, live=0, after helper destroyed the root. Expected: reject the local-root escape before dispatch (or explicitly reject this unsupported signature), not admit it and wait for use of the dead frame. The top-level Place-result rejection control passes but does not cover this internal function result.
2. `review_r5_record_place_token_refused`: otherwise legal beta unit/whole shape plus valid Place<i32> type and record `EscapedToken { token: Place<i32> }`. MIR **and ABC** admit the unused record type. Expected: reject its borrowed-token field, just like existing StrView/MutRef/RecordView restrictions. The table-only input isolates field-domain validation from runtime aggregate behavior; it does not claim an independently executed aggregate escape.

Established cause: ownership.rs:366..380 classifies only StrView/MutRef as tokens; LIST_PLACE around :1402 merely defines dst, without root provenance; Return around :1503 checks escape only for classified tokens. mir_verify.rs:525..527 excludes StrView/MutRef/RecordView fields but omits Place, so a container wrapper can hide the same token from the existing field gate.

Closure: make the supported Place domain and lifetime policy explicit and enforce it across direct/nested fields and function parameters/results. Keep valid local Place controls; replay a local-root return, a legitimate supported externally rooted return if any (otherwise controlled unsupported rejection), borrowed-token record field, and an ordinary scalar record positive control. Rejection must occur at the applicable MIR/ABC gate, not via a later InvalidHandle. Do not make all runtime-stale tokens admission errors: ordinary structural epoch/generation invalidation retains its existing controlled runtime semantics.

## Acceptance, SD and routing

| AC | Current disposition / evidence boundary |
|---|---|
| AC-01 | Prior first-case/parent-oracle evidence retained; no complete fresh pass inferred. Type-table closure fixed, but F-R5-02 still violates the type/borrow domain. |
| AC-02/03 | fail: cross-frame commit and borrow-escape integration. Eight local Place controls pass; prior transfer/reverse cleanup evidence remains within its inspected boundary. |
| AC-04 | fail: admitted program panics; provably escaping borrowed token and borrowed record field admitted. R4 bounds failure itself closed. |
| AC-05 | Prior transaction/resource/peak evidence retained; no new resource campaign, no complete AC pass. |
| AC-06 | Prior cache/recipe/compatibility evidence retained; complete remaining V-08/09/10/11/12 audit still pending. |
| AC-09 | Prior incremental/failed-candidate evidence retained, including this repair's prior-session control; no scalar campaign replay or whole AC pass. |
| AC-07 | fail: two contract findings remain. |
| AC-08 | pending merge, after final review pass only. |

V-01/02/11 are affected by these findings; local V-12 stale/fresh controls pass. Unchanged V-03..10/13 evidence remains bounded by earlier receipts rather than being discarded or promoted. Full remaining AC/V/SD audit is pending because work-owned invariants already fail; no global run was spent on known-blocked code.

SD-01/02 cannot publish complete typed-Place/lifecycle claims while these gaps remain. SD-03/04/05 profile/cache/recipe identities retain the prior outline assessment; SD-06's session transaction evidence remains, but this round does not grant the complete knowledge-delta pass. The draft's `must-intersect join` guard phrase also needs reconciliation with the recorded block-local implementation boundary before deposition; no new implementation scope or canonical edit is inferred. Exact target paths/impact metadata remain valid navigation; touched_goals=[] is justified by the bounded experiment, not production-goal closure.

Omission/workaround scan: Place was added to type/codec/instruction branches but not fully integrated into frame selection, borrowed-field domain and borrow-provenance analysis. Fixing only the old example would not close that shared invariant. These are in-contract blockers, not approved deferrable debt; recorded in KNOWN-DEBT-AND-RISKS.md.

Next: work in the same tree, reopen T-03/T-04/T-06; retain T-01/T-02/T-05/T-09, state executing 4/9. Diagnose the Place path/state matrix before readiness, implement supported behavior or explicit in-contract rejection, commit, and re-review affected evidence. No implementation edit, merge, archive or cleanup in this review.

Pending merge-owned gates remain frozen bare cargo t + cargo tv + the experimental workspace finite-contract checks and additional actual surface tiers. They run only after final acceptance/Spec review passes; cargo tf remains batch-only on main. Do not treat needs_fix or this closure of F-R4-01 as permission to land.

## Interrupted-review continuation — 2026-10-26

At the user's request, a continuation context reconciled the saved report, raw logs, replay inputs, main Plan receipt and risk register against the existing implementation tree. HEAD remains ad9a69ffa12919ac8812b18b78a0b481b6f319bb; tracked implementation is clean; the repair diff passes git diff --check. Cargo.lock, the retained R3 probe, frozen SD and all non-Plan input hashes match the saved baseline; both recorded D821 commits remain ancestors. The Plan hash is intentionally historical because review bookkeeping changed it. Source inspection independently confirms the owner-frame/precheck versus current-frame/commit mismatch, and the omitted Place token/field classification. Existing targeted behavioral logs are reused, not presented as newly executed tests. No additional implementation or regression run was needed to recover the interrupted closeout.

The R5 receipt and risk entries had already been saved. Final disposition remains needs_fix, executing 4/9, with F-822-R4-01 closed and F-822-R5-01/02 open. This continuation completes review reporting; it does not perform repairs, merge, archive, or cleanup.
