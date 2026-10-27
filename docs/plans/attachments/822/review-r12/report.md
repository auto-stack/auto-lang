# PLAN-822 final repair review R12 — needs_fix

2026-10-26; Plan revision 5, final review. Reviewer: current chat Codex, not the implementer, independent contract/source/behavior review; no subagent. Implementation writes paused; no repair within review.

## Bound baseline

Same tree D:/autostack/.wt/lang-822/auto-lang, branch plan-822-dev; reviewed code 90074e87f6c073868a0ee1f0642501bf383b7846; repair base b5c226214f3f32b41299df945585efabe40c6b28; original implementation base de93bacc0ad16622140b0adb4d36eee86fe7fc83. Main bookkeeping submission 392c7a7cd1e987d5326f98187ad838e15e249840 verified (execution_done7/9). Entry/final implementation HEAD match; tracked implementation clean. Existing untracked R3/R8 probes preserved, R12 temporary probes copied to this directory and removed.

Cargo.lock SHA256 37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873. Dependencies/schema unchanged. Main contract/Spec inputs hashed in input-hashes.txt; frozen SD copied to sd-frozen.md SHA256 4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739. No canonical Spec/ledger publication.

## R11 findings CLOSED

F-822-R11-01: original helper and production replays independently rerun. 4-byte helper input costs6 and succeeds with budget60; 4096-byte input charges4098 and rejects before allocation (minted0). In the real MIR→ABC→AV with/without-CLONE control, 4-byte increment9 (15→24), 4096-byte increment4101 (4107→8208), Success7/live0. Source now charges output Str bytes before its copy plus structural work; memory precheck is no longer the fuel substitute. replay-r11.log.txt and production-byte-fuel.log.txt.

F-822-R11-02: original zero-remaining-wall helper replay now returns failure with fuel0/minted0. Thread-local origin removed; remaining allowance is compared with elapsed since helper entry; entry check handles zero deterministically. No prior-call dependency or double subtraction. Existing positive-duration/byte-budget controls plus code check retained. The current evidence does not claim a measured full-dispatch wall-time campaign, and none is required merely to restate this repaired arithmetic.

Committed clone_ filter independently run: 12 passed/0 ignored, warning-free, including exact budget sides, partial abort cleanup, isolation, nested Variant and wall seam. clone.log.txt. GuardSelfTest valid control passes and skip/missing/zero/duplicate controls reject (guard.log.txt); source confirms stale R9 IDs removed and new R11 IDs added. Full semantics/compatibility scope runs reported by work retained as work evidence, not represented as freshly rerun here. Changed committed Rust files pass rustfmt --check --edition 2021; git diff --check b5c226214 HEAD clean.

No full tier/workspace suite, acceptance aggregate or unrelated performance/GC campaign. R11 closure does not mean the previously incomplete full acceptance audit has passed.

## F-822-R12-01 — P1: supported nested-value cleanup still uses host recursion

Classification: original collection integration omission uncovered while completing the previously pending cleanup matrix; NOT a regression introduced by the R11 byte/deadline repair, and not a claim that R11's two findings remain open. Affects AC-03/05, T-03/T-04/T-06, V-06/11 and SD-01/02.

Violated existing rule: Plan §5.1.6 explicitly requires graph validation AND CLEANUP to use explicit traversal (`表/图递归验证和清理显式遍历`), with owned cycles rejected and no hidden language depth limit. §5.2/§5.4 carry bounded cleanup/reserved fuel. This is an agreed implementation constraint protecting the full supported graph, not an added request to support huge values.

Established committed call paths in core/src/av.rs:

- release_frame_owned_lifo (:891) calls min_buffer_id for Variant/List/Record slots before reverse release.
- min_buffer_id (:934) directly calls itself on List elements and Variant payloads (and on Record descendants); the comment justifies this with 'containers one layer deep / table bounded'. The support matrix allows arbitrarily nested acyclic Variant payloads subject to existing shared budgets, so one-list-layer is not a host-call-depth bound.
- drop_owned_reverse (:972) directly calls itself for List elements and Variant payloads. The Variant arm clones the Rc payload handle and recurses; the parent var retains its payload while the recursive call runs, so nesting is not flattened by Rc or by this call shape.
- This path is used for normal Return and cleanup_owned_frames on trap/resource/cancel exits, as well as owned_kill_slot during moves/transfers. Plain drops of final nested Rc payload chains also need to be considered when replacing it; making only the minimum-id walk iterative is insufficient.

The Rc change correctly fixes snapshot acquisition recursion, but does not make release iterative. The visitor fix therefore cannot be used as evidence for this distinct cleanup consumer.

Independent domain/control evidence (not a reproduced stack crash): replay.rs::review_r12_nested_variant_cleanup_domain_control constructs 64 distinct legal acyclic Variant types, packs the chain via production instructions and returns its disc. Real verify_mir→encode_abc→verify_abc_bytes→execute admits and returns Success I32(0), steps10785/live0. cleanup-domain.log.txt. Thus nested Variant cleanup is a current reachable support path, not malformed input or an unsupported feature. The control succeeds and is NOT described as a failing runtime oracle. No stack-overflow crash, memory-limit breach or numerical failure threshold is claimed. The finding is established by the direct recursive call graph versus the explicit-traversal contract; table-bounded depth is not the agreed replacement for explicit traversal.

Minimal closing scope:

1. Convert cleanup traversal/order-key discovery and relevant last-owner payload release to bounded explicit work items, preserving reverse List element order, active Variant payload once-only semantics, ordinary shared transfer behavior and reserved cleanup fuel independent of normal cancellation/fuel exhaustion.
2. Audit both normal Return and fault/resource/cancel unwind, plus source-kill/Move/Copy/Call transfer sibling paths. Do not drain shared payloads or reintroduce the R3 list-move loss while flattening release.
3. Preserve the 64-layer valid production control, add meaningful nested owned-leaf and failure-exit controls with frozen reverse-event oracles/live0. Confirm the actual cleanup call graph is nonrecursive for the supported graph. No arbitrary depth cap, reduced type support, global regression or new scale/performance promise is requested.

Reproduction from the existing tree: copy replay.rs to probe/tests/review_822_r12.rs, then:

    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r12 review_r12_nested_variant_cleanup_domain_control -- --nocapture

This command demonstrates legal reachability and correct small-control output; use the code locations above to audit the unresolved explicit-traversal requirement. It is not a 'make the green test red' oracle.

## Acceptance / knowledge disposition

AC-01 retain prior first-case/parent-oracle evidence. AC-02 R11 clone accounting/deadline controls pass within recorded scope, other semantics retain prior evidence. AC-03/05 partial/fail due to explicit cleanup traversal; R11 byte/wall failures are closed. AC-04 prior type/Place/Variant-cycle admission evidence retained; verifier unchanged by repair. AC-06/09 prior cache/recipe/compatibility/incremental evidence retained within previous bounds; no new full pass inferred. AC-07 fail; AC-08 pending merge. Remaining final AC/V/SD audit boundaries stay explicit, not covered by test totals.

SD-01/02 lifecycle/cleanup completeness not approved until F-R12-01 closes. SD-03..06 retain prior review boundaries. Main metadata targets/touched_goals=[] limited experimental-scope rationale unchanged. Canonical Specs and ledger untouched.

Omission/workaround scan: the same false 'one container layer bounds recursion' rationale remained in the cleanup consumers after clone traversal was repaired. It violates the original contract and is not approved deferrable debt. Record in KNOWN-DEBT-AND-RISKS; reopen T-03/T-04/T-06, return executing4/9, preserve other tasks/history and all R11 closure evidence. Same-tree work should diagnose cleanup traversal/order/last-owner behavior together before re-review.

Pending merge-owned gates: after final code/acceptance/SD pass, freeze HEAD and run bare cargo t + cargo tv + independent experimental workspace finite-contract gates and actual additional touched tiers. cargo tf remains main-checkout batch-only. No implementation fix, merge, archive or worktree cleanup performed.
