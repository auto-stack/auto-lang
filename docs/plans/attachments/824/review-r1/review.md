# PLAN-824 final review R1 — needs_fix

- stage: review; review_scope: final; reviewer: Codex in this chat, independent of the supplied implementation handoff (no implementation edits in this review; no extra agent/session identity claimed).
- plan_revision: 7. Read the shared Plan on main, not the stale worktree copy. Input Plan SHA256 B6E360A894CBC6B2212ADE12CC8A0287558D6A119754BF73E3DC525C148C5F89; contract-decision SHA256 43B6AB684FB4191EAE9331F92406E481FA57B6C12393D6FE1B8683A7F2F67B70.
- worktree: D:/autostack/.wt/lang-824/auto-lang; branch: plan-824-dev; reviewed_commit: 2093675731fb1fe1b9aaa7291cdbbdee672bf9b5; base_commit/merge-base: c6c5ddb256089e6b628d7651cb9759664d15f22a. Main at 7e498e4ccbce5f0a7b7b8008d8303b170d6f373e. Implementation clean before/after inspection.
- dependencies: D821/D822/D828/D852 archived receipts inspected; consumed source is the base above (no unlanded 823 consumed). Sibling auto-ui b4fe0000e56cddaf7d3cf70e348431f2258aacde; auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4. Full source/Spec hashes: inputs.json. Existing parallel numeric identity conflict is still a closeout prerequisite, not resolved by this review.

## Reproduction

From the implementation worktree:

```powershell
cargo run --quiet --manifest-path D:/autostack/auto-lang/docs/plans/attachments/824/review-r1/probes/Cargo.toml --target-dir D:/autostack/.wt/lang-824/auto-lang/experimental/ac-av-probe/target/review824-r1
```

The probe uses the exact reviewed auto-av-core via path dependency, no copied production implementation. Raw output: replay.txt. It prints observed defects rather than failing intentionally; its exit 0 is NOT acceptance pass. Legal bool roundtrip is the paired control. The initial resource probe incorrectly used unit kind for execute and returned an admission error; corrected to new_dynamic whole kind before recording replay.txt. No product finding relies on that invalid setup. An initial nonexistent ExecLimits field was also corrected before successful execution.

## F-824-R1-01 [P1] Portable leaf descriptor and payload kind are not matched

Existing rule: §5.1/§5.5 and V-05/08/12 require untrusted inner bytes to be validated before granting typed access. portable.rs:529..552 pairs every scalar value with arbitrary decoded node_ty; :755 constructs DynValue using from_parts without shape admission. Unlike containers, scalar leaves never check kind equivalence.

Replay: freeze Bool(true); mutate inline type byte at offset 16 from TY_BOOL=0 to TY_I32=1; recompute FNV trailer. Expected invalid/type rejection. Observed Ok(DynValue { value: Bool(true), ty: I32 }); take_as(I32) then returns Ok(Bool(true)). This is an actual checked-type invariant failure, independent of cache checksum. The root and nested leaf paths share the same missing check. Legal bool input succeeds.

Close oracle: reject all scalar/descriptor cross-kind combinations, at root and under a record/List/active variant payload, including a correctly recomputed checksum; preserve legal bit-exact roundtrip and wrong-type take source preservation. Relevant work: T-03/T-05/T-10; AC-02/03/04/06/10/11 and dependent T-06/T-11 evidence. No unrelated whole-suite replay needed during repair.

## F-824-R1-02 [P1] RestoreContext cannot validate current field/payload types or inactive nominal dependencies

Existing rule: §5.1.4–5, §5.4, V-05/08/11 require full stable type/current layout admission on restore. RecordBinding only contains owner/name/field NAMES/index; VariantBinding only arm names/has_payload/index. portable_bindings_of (dynamic_fixture.rs:234..260) discards all field and arm TypeRefs. portable.rs:573..588/:654..666 only compare that reduced shape, and nominal checks occur only when the VALUE tag visits a record/variant.

Replay A (direct production decoder): an empty List<Record gone-owner/Gone{x:i32}> frozen legally, then unfreeze with EMPTY current bindings. Expected unknown-type rejection; observed success. No record value exists to trigger the nominal check. Same invariant includes inactive variant arms and unused nested type parameters.

Replay B demonstrates the binding information loss: freeze owner/R{x:i32}; restore against owner/R field name x, current positional index 77. Observed a RecordValue typed 77 holding I32(42). A current R{x:bool} produces exactly this same binding through the real adapter; that binding cannot even express the changed field type. This probe does not claim it supplied a bool TypeRef to RecordBinding: its inability to carry one IS the established cause. A caller-supplied types_fp/cache-key change does not fix direct unfreeze or guarantee ctx_module matches that key.

Close oracle: bindings/current schema must validate complete recursive type identity (including applicable revision/layout facts), even when List is empty or an arm is inactive; add same-name/same-fields changed field TYPE and changed variant payload TYPE controls and current-table/store restore controls. Legal same-schema restore into a fresh positional index stays supported. T-03/T-05/T-10 and dependent T-06/T-11; AC-02/04/06/10/11. Actual current-table rejection is required, not merely asking a different cache key and observing a miss.

## F-824-R1-03 [P1] Dynamic type DAG expansion bypasses logical work and memory accounting

Existing rule: §5.2/§5.4 require type analysis, traversal and temporary construction to use bounded work/data budgets and cancellation, not just a depth cap. av.rs:574 resolves a compact shared type DAG into an owned DynType TREE, recursively cloning names and expanding both branches each time. dyn_whitelisted (mir_verify.rs:933) similarly traverses every path without a charged work budget. DynIs charges one step after resolution. contract.rs:674 explicitly excludes descriptors from deep_size.

Replay: a legal whole dynamic module packs i32 then DynIs against 17 nested variant definitions, each with two arms sharing the preceding type. ABC is only 721 bytes, depth 18 (well below 256). verify_abc_bytes grants the credential. Runtime expands 262143 DynType nodes (~20971440 bytes of DynType inline storage alone, excluding names/Vec allocation), then returns Success Bool(false), steps=8, peak_slot_bytes=160, under max_steps=100/max_slot_bytes=4096. This exceeds the execution 8MiB policy with invisible temporary work. The ~20MiB is a structural allocation lower-bound calculation, not a measured process RSS claim. No hostile huge input or unsupported type is involved.

Close oracle: bound/charge type expansion and comparisons before excess work/allocation, with cancellation/error classification and cleanup; compact DAG sibling controls on both sides of the existing budget. Cover DynPack/DynIs/DynTake and corresponding whitelist/portable type traversal where the same mechanism applies. A new syntax/depth exclusion cannot silently replace the approved budget. T-03/T-04/T-10 and dependent T-06/T-11; AC-02/03/05/10/11. Re-run these targeted probes first, unified regression remains merge-owned.

## Coverage and health

Independent command in experimental/ac-av-probe:
`cargo test -p auto-av-probe --test av_dynamic_824_semantics --test av_dynamic_824_incremental`
Result: semantics 20/20, incremental 5/5, ignored=0. These existing tests remain green in the presence of all three defects. Incremental baseline campaign also executed through its existing test. Compiler emitted unused import warning at probe/tests/av_dynamic_824_incremental.rs:14 (verify_mir); the handoff's zero-warning claim is false for test targets. Remove it and check the affected test targets during repair. `git diff --check master...HEAD` clean. No implementation fixes made.

| Acceptance | R1 evidence/result |
|---|---|
| AC-01 | partial: bounded first-case/parent-oracle artifacts inspected, existing semantic positives pass; no final profile-wide pass |
| AC-02/03/04 | fail: F01/F02 typed invariant + F03 budget gap |
| AC-05 | fail: F03 invisible type allocation/work; existing allocator/worker sample tests do not cover it |
| AC-06 | fail: F01/F02 restore admission; existing cache-key miss/structural-corruption controls insufficient |
| AC-07 | fail: unresolved findings and test warning |
| AC-08 | pending merge; not executed |
| AC-09 | partial: 5 existing incremental tests pass; broader applicable resource/schema controls require affected repair verification |
| AC-10 | fail: same-source module contains F01/F02/F03 |
| AC-11 | fail: D06/D07/D08/I04 and applicable resource evidence do not close the invariant; existing green mapping is insufficient |

V-01/02/03 semantic positives and V-13 existing controls independently replayed. V-04/05/08/11/12/14/15 restore paths inspected with independent counterexamples. V-06/07 budget paths inspected and F03 reproduced. V-09 compatibility and V-10 parent-oracle implementation/script inspected; prior handoff runs not re-executed in full because implementation already needs repair. Parent peak sampling, every allocator fault position, fresh-process restoration and complete prior-consumer matrix are not independently certified by this review; existing evidence preserved with its original boundary, affected paths must be refreshed after repair. No full acceptance/whole-tier gate was spent on known-red code.

## Spec delta and disposition

Frozen proposed SD-01..07 copy: spec-delta-frozen.md, hash in inputs.json. Targets/frontmatter are present; canonical Specs were read only. Current delta cannot claim safe typed portable restoration/current-type validation or resource-bounded dynamic type operations until F01..03 close. touched_goals stays []: this experimental profile does not add/retire a repository goal ID; no invented IDs. No live ledger/Spec publishing in review.

Outcome needs_fix, status executing. Reopen T-03/T-04/T-05/T-06/T-10/T-11 (3/11 completed: T-01/T-02/T-09 retained within their bounded evidence). T-07/T-08 remain incomplete; semantic plan_revision stays 7. Work repairs in the same tree, commits, then requests affected re-review with closure table. Bare cargo t + applicable tv + new-workspace closeout gates remain pending, owned by merge after final review; tf batch only on main. Numeric collision with 823 must be reconciled against the actual integration baseline and affected bytes/consumers reverified before landing; future renumbered code is not covered by this SHA.
