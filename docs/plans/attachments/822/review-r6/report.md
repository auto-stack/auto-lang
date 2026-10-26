# PLAN-822 final repair review R6 — needs_fix

2026-10-26, Plan revision 5. Review scope: final. Reviewer: current chat Codex, separate from implementation of this repair; inspected the committed diff, contract and independently extended replay. No delegated agent. This context also recovered the prior R5 review; conclusions are reconstructed from code and behavior rather than the work summary.

## Baseline

- Worktree: `D:/autostack/.wt/lang-822/auto-lang`; branch `plan-822-dev`.
- Reviewed commit: `352cbc5c4e1afe4dd1a515e856d4986503506a22`.
- Repair base: `ad9a69ffa12919ac8812b18b78a0b481b6f319bb`; implementation base: `de93bacc0ad16622140b0adb4d36eee86fe7fc83`.
- Main bookkeeping HEAD at entry: `2e09547eea9fe24d2fee5e81f7509c0dc40d7897`. Shared Plan read on main, entering execution_done 7/9.
- During review main advanced to `bc094d132da6cab6853d030a743a6ab2b477b682` for unrelated PLAN-852 bookkeeping. The PLAN-822 implementation HEAD did not move. All recorded non-Plan canonical/contract/SD hashes were rechecked and match R5; affected evidence remains valid. No unrelated changes were staged or reverted.
- Cargo.lock SHA256 remains `37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873`. Existing D821/D828 dependency/window evidence is retained; this repair changes no dependency manifest or lockfile. Current canonical Spec/contract/SD hashes are recorded in `input-hashes.txt`; the Plan hash is before R6 bookkeeping.
- Proposed SD frozen in `sd-frozen.md`, SHA256 `4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739`. No canonical Spec or ledger publication.
- Entry and final HEAD match; tracked implementation clean. Existing untracked R3 probe remains untouched, SHA256 `3F84BA196CFDDF2853D5102D38FEA83FF45DBC4E09955A65738A0F10988306D8`. Temporary R6 probe was saved as `replay.rs` and removed. No product implementation edits.

## Checks and R5 disposition

The work diagnosis correctly identifies the frame mismatch and missing Place token classification. Its five added tests cover the original examples, but the new separation of retained provenance from active borrow liveness is incomplete at dataflow joins and alias/call operations.

| Check | Scope / observed result |
|---|---|
| Original independent R5 replay, filter `review_r5` | 5 passed, 0 ignored. Cross-frame write returns 9, read returns 7, local-root same-block return and record{Place} rejected by MIR and ABC. Entry-result rejection remains an accepted control. |
| Committed semantics, filter `place_` | 13 passed, 0 ignored. Includes stale/fresh, type binding, cross-frame writes/reads, local escape/record refusal and supported external-param roundtrip. |
| Independent new replay, filter `review_r6` | 3 passed / 2 failed, 0 ignored. See findings below. Valid external-root return across a jump returns 7/live=0; cross-block read returns 7; direct-place retirement/update returns 9. |
| Repair diff whitespace check | `git diff --check ad9a69ffa HEAD` clean. |
| Changed committed Rust files format | `rustfmt --check --edition 2021` on av.rs, mir_verify.rs, ownership.rs and semantics test: exit 0. Initial mistaken edition-2024 check is retained separately in fmt.log.txt; its reserved-gen/import-order output is a review invocation error, not a product defect. Actual manifests specify edition 2021. |

Targeted builds emitted no compiler warnings. No whole-tier cargo t/tv/tt/tb/tu/taa, full workspace suite, aggregate acceptance, memory campaign or performance replay was run. Known work-owned blockers do not justify a global run.

**F-822-R5-01 CLOSED** on this baseline: the commit path now locates the owner frame by the same activation identity as precheck/read resolution. Original admitted caller/helper write executes successfully with caller result 9 and balanced arena cleanup. The complete new owner-frame path was inspected; there is no interleaved dispatch between successful precheck and commit.

**F-822-R5-02 PARTIAL**: original same-block local escape and record-field examples are fixed; the local-root provenance invariant is still violated across basic blocks (F-822-R6-01). Adding Place to the existing active-token branches also introduces F-822-R6-02. Original-example green counts do not close the full finding.

## F-822-R6-01 — P1: local Place provenance is lost across a basic-block edge

Classification: residual of F-822-R5-02, not a new language feature. Affects AC-02/03/04, T-03/T-04/T-06, V-01/02/11 and SD-01/02.

Existing rule: Plan §5.1.6 requires borrow escape rejection; R5 closure requires local-root Place returns to fail at MIR/ABC admission, while legitimately external roots remain supported. Moving an otherwise identical Return into a successor block does not make the local root outlive its frame.

Bound input: `review_r6_local_place_escape_across_jump_refused` in replay.rs. It uses the prior independent nested-callee escape shape: i32 entry calls a helper returning Place<i32>; helper creates List<i32>[7], mints its element Place in slot 4, then executes `Jump { target: 1 }`; block 1 returns slot 4. Entry reads the returned Place. Only the added control-flow edge distinguishes it from the now-refused same-block R5 input.

Expected: MIR and real ABC admission reject the locally rooted return before dispatch; no unchecked credential should permit the escape. Observed: both gates succeed (`mir-ok=true, abc-ok=true`); real execution returns `Trap { kind: InvalidHandle }`, live=0 after the helper root dies. The controlled runtime trap does not satisfy the promised admission rejection. The runtime does not panic in this probe.

Established cause: LIST_PLACE at ownership.rs:1412 records token_roots but deliberately does not set tokens_live. The predecessor merge at :524 copies roots only if the token is live; the retained Place's roots are therefore discarded at the successor. Return at :1520 classifies Place as a token but sees no local roots and permits escape. `roots_eq` at :1604 likewise ignores root changes for non-live tokens. This is an incomplete separation of retained provenance and active borrow state, not a bounds/type-table issue.

Accepted controls: original same-block nested escape is now rejected; a caller-owned Place parameter returned through an actual helper Jump succeeds with caller read=7/live=0; cross-block ordinary read also succeeds. These demonstrate that the failing input reaches supported CFG execution and that the correction must distinguish root provenance, rather than reject every multi-block Place.

Closure: preserve retained provenance through relevant predecessor merges and worklist equality independently of exclusive/live-borrow state. Replay same-block and cross-block local returns at both gates, a representative branch merge with local/external roots (including predecessor ordering), and supported external-param return/read. Audit Copy/Call propagation and the existing analysis accounting when integrating the new retained state. Ordinary epoch/generation invalidation remains a controlled runtime event, not a requirement to statically reject every stale token. Limit checks to this provenance/lifetime mechanism; do not attach full regression or performance campaigns.

## F-822-R6-02 — P2: Copy promotes a retained Place into an active borrow

Classification: repair regression from adding Place to is_token without adapting the existing Copy/Call branches. Affects AC-02/03, T-03/T-04/T-06, V-02/11/12, SD-01/02.

Existing rule: the R5 repair explicitly preserves the supported retained-reference policy: minting Place does not establish an active borrow, and ordinary list updates retire old Places by epoch/element generation. A Copy alias of that same retained reference must preserve this policy; it does not introduce a new exclusive-borrow operation. This follows the declared copy/borrow responsibility invariant (AC-03), not a request for additional container syntax.

Bound inputs: `r6_mutate_after_place(false/true)` and the two retirement tests in replay.rs. Both programs build List<i32>[7], mint Place<i32> slot 4, set the existing element to 9, then read by a fresh list index. Neither dereferences the retired Place. The second program adds only `Copy { dst: 5, src: 4 }` before ListSet.

Expected: both admitted programs return 9, with any retained tokens subject to the same generation retirement policy. Observed: direct mint passes MIR/ABC and returns 9; adding Copy makes both gates reject with `mir.list.place-borrowed` / active borrow on the root. It is an erroneous rejection, not runtime corruption.

Established cause: Copy at ownership.rs:629..632 now matches Place via the expanded is_token classification, propagates roots and unconditionally sets tokens_live. ListSet's root_has_live_token gate sees the copied token as an active borrow, unlike the original minted token. Call's token-result branch at :689..703 has the same unconditional activation; that sibling risk is source-inspected, not independently reported as an executed failure.

Closure: make the retained-reference/active-borrow policy consistent for supported aliases and returns. Replay direct versus copied Place update, and a supported externally rooted helper roundtrip followed by list update; retain stale-token failure and fresh-index success controls. Fixing F-R6-01 by simply setting all Places live would mask provenance loss while reproducing this unwanted rejection. No global or performance campaign is required for closure.

## Acceptance, knowledge delta and next action

| AC | Disposition / evidence boundary |
|---|---|
| AC-01 | Prior first-case/parent oracle evidence retained; no complete new pass. Original record-field/type-table gaps are fixed within the tested boundary. |
| AC-02/03 | fail: retained provenance still lost at CFG edges; Copy changes the borrow policy. Cross-frame write and the 13 Place controls pass. Prior move/reverse-cleanup evidence retained. |
| AC-04 | fail: provably local-root escape admitted by both real gates, only failing after return. R5 panic and direct record-field defects closed. |
| AC-05 | Prior transaction/resource/peak evidence retained within its recorded boundary; no new campaign or complete fresh pass. |
| AC-06/09 | Prior cache/recipe/compatibility/incremental evidence retained; remaining full AC/V audit pending. No unrelated replay from a changed HEAD alone. |
| AC-07 | fail: the above contract findings remain. |
| AC-08 | pending merge; no landing authorized by this review. |

V-01/02/11/12 are affected; prior V-03..10/13 evidence remains bounded by earlier receipts, neither discarded nor promoted to a full pass. Further final AC/V/SD auditing is deferred once the reproducible in-contract blockers are established; no pass is granted for unexamined required areas.

SD-01/02 cannot publish complete Place lifecycle/admission rules while the above gaps remain. SD-03..06 retain the prior review boundary and current exact target paths; touched_goals=[] remains justified by the limited experiment. The unchanged SD-02 guard phrase (`must-intersect join`) still requires reconciliation with the earlier recorded conservative/block-local gate boundary before deposition. No canonical/ledger edits were made. Current input hashes and frozen SD are durable evidence, not authorization to weaken the contract.

Omission/workaround scan: R5 correctly separated minted Place provenance from active borrowing at creation, but did not carry that distinction through predecessor merge/equality, Copy and token-return handling. Both findings are within the existing contract; risk registration does not authorize deferral. Reopen T-03/T-04/T-06, retain T-01/T-02/T-05/T-09, return to executing 4/9. Same-tree work should diagnose the complete retained-provenance path and add permanent distinguishing controls, commit, then submit affected criteria for re-review.

Pending merge-owned gates remain bare cargo t + cargo tv + the experimental workspace complete finite-contract checks and any additional actual surface tiers, only after final acceptance/Spec review passes on a frozen commit. cargo tf remains batch-only on main. No merge, archive, cleanup or implementation repair occurred.

## Reproduction

Copy this directory's replay.rs into the existing tree as `experimental/ac-av-probe/probe/tests/review_822_r6.rs`, then run from that tree:

```text
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r6 review_r5 -- --nocapture
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r6 review_r6 -- --nocapture
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_collections_822_semantics place_ -- --nocapture
```

Raw durable logs: replay-r5.log.txt, new-replay.log.txt, place.log.txt, fmt-2021.log.txt. New-replay's final five-case result supersedes earlier one/four-case exploratory invocations; no cases are counted twice. replay.rs includes inherited earlier helper/tests for reproducibility; only the stated filters were run in this round.
