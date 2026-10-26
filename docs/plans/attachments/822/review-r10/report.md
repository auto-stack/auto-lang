# PLAN-822 final repair review R10 — needs_fix

2026-10-26; Plan revision 5; final review. Reviewer: current chat Codex, not the repair implementer; independent artifact/source/behavior reconstruction, no subagent. Work remained paused, no implementation repair performed.

## Bound baseline

Same tree D:/autostack/.wt/lang-822/auto-lang, branch plan-822-dev, reviewed commit 9e65e606d411693da2d5e21053758a86e0977c52; repair base 7402022b5976531b6c685104d87d7354111a0766; original implementation base de93bacc0ad16622140b0adb4d36eee86fe7fc83. Entry/final HEAD identical, tracked implementation clean. Existing untracked review_822_r3.rs and review_822_r8.rs preserved. Temporary R10 probes were copied to this directory and removed from the implementation tree.

Cargo.lock SHA256 37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873; dependencies and wire schema unchanged. Main Plan/contract/Spec hashes in input-hashes.txt, frozen proposed SD sd-frozen.md SHA256 4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739. No canonical Spec or ledger edits. Relevant retained evidence still has the same declared boundary, not elevated to a final pass.

## R9 disposition / targeted evidence

F-822-R9-02 CLOSED in the inspected graph domain: original independent self-cycle is now refused at MIR with mir.type-cycle and at ABC; valid three-layer nested Variant still admits/clones/executes disc0/live0. replay-r9.log.txt. Code removes the Record-only early skip, and both Record fields and Variant payload edges reach the shared color DFS. Work's new indirect-cycle test is retained as supporting evidence; this review does not pretend its raw run was freshly repeated.

F-822-R9-01 remains PARTIAL: explicit top-level loops and actual cancel/wall polls were added, but nested Variant snapshots are not shallow and the new runtime oracles do not reach the claimed path. See F-822-R10-01 below. R8 original isolation closure retained.

Committed clone_ filter: 8 passed, 0 ignored, including the original 5 controls plus nested/cancel/wall controls. However the committed bin emits an unused-variable warning (F-822-R10-02). clone-controls.log.txt. Changed committed Rust files pass rustfmt --check --edition 2021; git diff --check 7402022b5 HEAD clean.

Independent oracle mutation: a TEMPORARY COPY of the committed semantics test file replaces `instrs.push(MirInstr::Clone { dst: 2, src: 0 });` with a comment. Product files and implementation remain unchanged. Selecting only mid_traversal still gives 2 passed/0 ignored. no-clone-controls.rs and no-clone-controls.log.txt. The original test source is bound by the reviewed commit. Only named filters ran; no aggregate acceptance, full tier/workspace suite, unrelated memory/GC/performance campaign.

## F-822-R10-01 — P1: R9 clone traversal/cancellation invariant not closed

Classification: residual F-R9-01 plus repair evidence gap; affects AC-02/03/05, T-03/T-04/T-06, V-03/11 and SD-01/02. Existing Plan §5.1.6 requires explicit bounded traversal and §5.4/V-11 require cancellation during meaningful clone work. This is not a new scale or latency promise.

Established source chain:

- av.rs::clone_owned_value says its owned snapshots are shallow handle bumps. In fact it creates the initial worklist with val.clone(), clones each Variant payload while expanding children, and clones nodes[id].src again in assembly.
- contract.rs::Value derives Clone; Value::Variant contains VariantValue BY VALUE. VariantValue also derives Clone and carries Option<Box<Value>>. Box<Value>::clone recursively clones the contained value, so every Variant snapshot recursively duplicates its whole payload subtree. Variant is unlike Rc<RecordValue> or ListHandle. There is no custom iterative Clone here.
- Nested Variant is legal, independently confirmed by the retained three-layer production control. Thus this new two-phase visitor still invokes data-depth-dependent host recursion before and between its polls, and retains copied suffix trees in nodes. A chain of depth d causes quadratic aggregate snapshot payload copying rather than shallow O(d) nodes. This is direct control/type-layout evidence; no stack-overflow crash or measured memory-cap breach is claimed.
- The initial val.clone() occurs before the first helper poll. Child-expansion loops also clone snapshots without polling each pushed child. The local visited counter counts nodes; it does not enforce remaining execution fuel. The helper starts a fresh wall timer using the full limits.wall rather than the original execution deadline. These limitations must be reconciled with the claimed shared bounded traversal; inserting outer loops alone is not sufficient.

Oracle defect is independently demonstrated, not hypothetical: both committed mid_traversal tests execute 1,500 Const/ListPush pairs before CLONE. They start with cancel=true or wall=0. The ordinary dispatch loop can observe those conditions during construction. Removing the CLONE instruction in a test-only copy STILL passes both tests. Therefore these results cannot demonstrate entry into the helper, interruption of either phase, or cleanup of any newly cloned partial assembly. The work claim 'mid-clone at first in-loop poll' is not established by these inputs. See no-clone-controls.log.txt (2/2 green despite no CLONE).

Expected closing oracle and minimal work scope:

1. Truly nonrecursive, shallow structural work items for supported nested Variant/List/Record values, including acquisition and release paths used by the clone traversal. Do not hide recursion in derived Value clones or add a language depth cap. Preserve source order, isolation and copy/move semantics.
2. Carry actual remaining execution deadline/fuel and perform bounded polls within meaningful work, with partial allocations cleaned on abort. Audit temporary snapshot lifetime/memory, not merely the final arena live count.
3. A production-path cancellation/wall test must prove that CLONE was entered, relevant node work/partial assembly occurred, and interruption happened there; include positive controls and a no-CLONE/disabled-helper-poll negative control that fails the claimed oracle. A pre-raised cancel or zero wall is usable only after preparing input without triggering outer dispatch first, with an explicit helper-entry witness.
4. Re-run original isolation/nth-fault/slot-budget controls and representative nested supported shapes. No whole-tier or global campaign is required for this repair.

These are the existing R9 closing conditions, not new feature scope. Do not mark F-R9-01 closed from the current test total.

## F-822-R10-02 — P2: committed semantics test emits an unhandled compiler warning

AC-07/T-06 mandatory health check requires zero unhandled compiler warnings. av_collections_822_semantics.rs:3122 declares c0 in nested_variant_clone_control and never uses it. Fresh targeted build of the ORIGINAL committed bin reports `warning: unused variable: c0`; clone-controls.log.txt. This is not just a warning introduced by the test-only no-clone mutation. Work's zero-warning statement is contradicted by the original build output.

Close by removing or meaningfully using the unused binding and confirming warning-free targeted compilation; no blanket lint suppression, no full-tier run needed for hygiene alone.

## Acceptance / SD / routing

AC-01 prior first-case/parent oracle evidence retained. AC-02/03 partial/fail due to residual bounded traversal; AC-04 R9 Variant-cycle defect closed within inspected direct/nested graph paths, other admission evidence retains prior scope. AC-05 partial/fail: actual clone interruption/partial cleanup oracle not delivered. AC-06/09 retain prior cache/recipe/compatibility/incremental evidence and unresolved full-audit boundaries, no unrelated replay. AC-07 fail; AC-08 remains pending merge. Remaining full AC/V/SD audit is not asserted complete on known-blocked code.

SD-01/02 cannot claim universally nonrecursive bounded clone/cancellation yet. SD-03..06 retain earlier disposition. Metadata targets and touched_goals=[] scope rationale unchanged. No semantic contract revision or canonical publication. Omission/workaround scan identifies a false shallow-clone assumption about a newly owned value kind and nondiscriminating cancellation tests; both are in-contract required work, not approved deferrable debt.

Record findings, reopen T-03/T-04/T-06, return executing4/9; preserve T-01/T-02/T-05/T-09 and historical evidence. Next: same-tree work must repair root cause and meaningful closing oracles, commit, then re-review affected criteria and finish remaining AC/V/SD audit.

Pending merge-owned gates: after final acceptance/SD pass, freeze HEAD and run bare cargo t + cargo tv + independent experimental workspace finite-contract gates and actual additional touched tiers. cargo tf remains main-checkout batch-only. No implementation fixes, merge, archive or cleanup.

## Reproduction

Copy replay.rs and no-clone-controls.rs into the same tree's probe/tests as review_822_r10.rs and review_822_r10_noclone.rs. From that tree:

    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r10 review_r9_ -- --nocapture
    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r10_noclone mid_traversal -- --nocapture
    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_collections_822_semantics clone_ -- --nocapture

The temporary no-clone copy inherits the original c0 warning. Only named tests were selected, no inherited campaign suites ran.
