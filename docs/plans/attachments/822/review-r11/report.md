# PLAN-822 final repair review R11 — needs_fix

2026-10-26, Plan revision 5, final review. Reviewer: current chat Codex, not the repair implementer, independently reconstructed from main contract, committed implementation and targeted counterexamples. No subagent; implementation writes paused.

## Baseline and R10 closure boundary

Same implementation tree D:/autostack/.wt/lang-822/auto-lang, branch plan-822-dev, reviewed commit b5c226214f3f32b41299df945585efabe40c6b28; repair base 9e65e606d411693da2d5e21053758a86e0977c52; original implementation base de93bacc0ad16622140b0adb4d36eee86fe7fc83. Entry/final HEAD identical, tracked implementation clean. Existing untracked R3/R8 probes preserved; new R11 probes copied here and removed from tree.

Cargo.lock SHA256 37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873. Dependency manifests, wire schema and verifier unchanged by repair. Main-checkout inputs bound in input-hashes.txt; frozen proposed SD copied as sd-frozen.md SHA256 4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739. No canonical Spec/ledger edits.

F-R10-02 CLOSED: original committed semantics bin builds without the c0 warning. Changed committed Rust files pass rustfmt --check --edition 2021; git diff --check 9e65e606d HEAD clean.

F-R10-01 PARTIAL: Variant payload now uses Rc<Value>, so derived snapshot Clone is a shallow handle bump rather than recursively cloning Box payload suffixes. Direct production-helper cancel/fuel seams now distinguish helper execution from dispatch; the earlier no-CLONE false-positive tests have been removed. However shared byte fuel was removed rather than preserved, and wall deadline handling remains wrong. The work's stated wall evidence boundary is explicitly incomplete; cancel coverage cannot establish the correctness of different time arithmetic.

Independently ran committed clone_ filter: 9 passed, 0 ignored, warning-free (clone-controls.log.txt). It covers the valid clone/isolation/nested/fault/slot-budget/seam paths named in that log. R9 cycle admission code unchanged; retain its prior independently confirmed closure, not rerun unrelated families. Two new targeted probes below fail; no full tier/workspace/acceptance aggregate, no unrelated performance or GC campaign.

## F-822-R11-01 — P1: repair removes output-byte-scaled clone fuel

Classification: repair regression and unresolved R9/R10 bounded-work invariant, AC-02/03/05, T-03/T-04/T-06, V-03/11, SD-01/02. Plan §5.4 requires large clone/construction/cleanup charging by data volume. Inherited canonical docs/specs/auto-av/place-ownership.md explicitly says Const/Clone (bytes + fields) count fuel. Merely checking deep byte size against the memory limit is not charging execution fuel.

Established source: CLONE arm removes `st.steps += new_size as u64 + visited` and replaces it with helper node counting. clone_owned_value::poll increments shared fuel by exactly one per planning/assembly node. Value::Str assembly copies h.bytes() into try_alloc without charging their length or polling per byte chunk. new_size remains only a memory precheck. A single large Str therefore costs two helper node units regardless of copied bytes; the same regression reaches old av.owned callers because CLONE uses this shared helper.

Durable independent production replay, production-replay.rs::review_r11_production_clone_byte_fuel: for lengths 4 and 4096, construct a legal collections module containing Const owned Str, optional Clone to matching owned destination, scalar result 7. Both paths run real verify_mir → encode_abc → verify_abc_bytes → execute, with the only module difference being presence of CLONE. Subtract the no-clone baseline to isolate clone cost from initial Const materialization. Results:

| Str output bytes | no CLONE steps | CLONE steps | clone increment |
|---|---:|---:|---:|
| 4 | 15 | 20 | 5 |
| 4096 | 4107 | 4112 | 5 |

All executions return Success I32(7), live0. Expected data-scaled increment for the 4096-byte copy is at least 4096; observed5. Test fails exit101. production-byte-fuel.log.txt. This is an output-volume accounting oracle, not a performance threshold or a request to accept larger inputs.

Second independent helper probe replay.rs::review_r11_byte_scaled_clone_fuel: the actual production helper called directly, detached Str inputs of 4/4096 bytes, fresh arena, same max_fuel60. Both succeed with fuel2 and mint one buffer; 4096-byte output should exhaust the established byte-scaled budget. byte-fuel.log.txt, exit101. Legitimate 4-byte control succeeds. The direct helper has no dispatcher that could obscure the charge.

Closing scope: retain enforceable shared byte + structural charging with bounded cancel/wall points before/during data copies; do not restore an unenforceable after-work check or replace memory budget for fuel. Assert small/large same-shape controls, exact budget sides, cancellation/partial cleanup and the original isolation/fault controls. Inspect Str/Record/List<owned>/Variant payload sibling paths and existing owned profile use of the shared helper. Targeted checks only; no whole suite attached to this finding.

## F-822-R11-02 — P1: clone wall accounting mixes elapsed and remaining time

Classification: residual R10 deadline defect, AC-03/05, T-04/T-06, V-07/11, SD-01/02. R10 required the actual remaining deadline, not an unrelated or restarted allowance. The current implementation is not validated by sharing an if-block with cancellation.

Established production arithmetic: dispatch stores segment start s into thread-local SEGMENT_STARTED_AT; at CLONE entry time t it computes `wall_left = W - (t-s)`. Helper later compares cumulative elapsed `now-s` against this ALREADY REDUCED wall_left. Thus elapsed before CLONE is subtracted twice. Example math (not a measured timing claim): with W=100ms, helper entered at elapsed60ms, remaining40ms, a helper poll at elapsed61ms rejects although the original deadline has39ms remaining. Use one coordinate system: absolute deadline, or elapsed since helper entry compared with remaining allowance; do not compare cumulative elapsed to remaining duration.

The exposed production seam additionally has no usable wall clock when called without prior dispatch on the thread: SEGMENT_STARTED_AT=None falls back to elapsed0. replay.rs::review_r11_zero_remaining_wall passes 1300 scalar elements, remaining wall Duration::ZERO, cancel=false, unlimited fuel on a fresh test thread. It traverses2602 nodes (crossing poll points), SUCCEEDS, mints a list buffer, rather than returning ExecutionWall. wall-seam.log.txt, exit101. Boundary: this dynamic result demonstrates the direct production-helper seam, not a timed full-dispatch run; the production double-subtraction is independently established from the real caller/callee expressions above. A hidden thread-local state is not an explicit deadline contract for the seam.

Closing scope: one explicit deadline/time origin passed through dispatch/helper; meaningful positive remaining-time and expired-time controls exercising actual helper polls, with independence from previous calls/thread-local leftovers. Include proof of helper entry and preserve partial cleanup. Avoid wall-time flake by a deterministic clock/deadline observation where feasible; do not drop the wall oracle because cancellation uses a neighboring branch. No general performance matrix or arbitrary runtime latency requirement added.

## Acceptance / SD / route

AC-01 retain prior first-case/parent oracle evidence. AC-02/03/05 partial/fail due to clone byte charging and deadline handling. AC-04 prior Variant cycle/admission evidence retained, unchanged verifier; no new full admission audit claimed. AC-06/09 retain prior cache/recipe/compatibility/incremental evidence within recorded bounds. AC-07 fail; AC-08 pending merge. Remaining full AC/V/SD audit is not declared complete on known-blocked code.

SD-01/02 cannot claim full data-scaled bounded/cancellable clone; other SD retain prior disposition, targets/touched_goals=[] rationale unchanged. Rc snapshot representation itself is consistent with shallow acquisition, but it does not discharge execution accounting. Review does not publish canonical Specs/ledger or weaken the contract.

Omission/workaround scan: repair traded the prior byte charge for node-only charging and substituted cancel-seam coverage for an untested wall calculation. Both are required current-contract fixes, not deferrable debt. Record in KNOWN-DEBT-AND-RISKS; reopen T-03/T-04/T-06; executing4/9, preserve T-01/T-02/T-05/T-09/history. Same-tree work should repair those mechanisms and provide discriminating targeted oracles before another final review.

Pending closeout: after final acceptance/SD pass, merge freezes HEAD and runs bare cargo t + cargo tv + independent experimental workspace finite-contract gates and actual additional touched tiers. cargo tf remains main-checkout batch-only. No implementation fixes, merge, archive, or worktree cleanup.

## Reproduction

Copy replay.rs and production-replay.rs to the existing tree's probe/tests as review_822_r11.rs and review_822_r11_production.rs, then from that tree:

    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r11 review_r11_byte_scaled_clone_fuel -- --nocapture
    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r11 review_r11_zero_remaining_wall -- --nocapture
    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r11_production review_r11_production_clone_byte_fuel -- --nocapture
    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_collections_822_semantics clone_ -- --nocapture

Only named filters ran; production-replay.rs includes inherited committed tests for reusable builder helpers, not run/count inflation.
