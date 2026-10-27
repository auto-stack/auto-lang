# PLAN-822 final repair review R13 — needs_fix

2026-10-26; Plan revision5; final review by current chat Codex, not the implementer; independent source/contract/behavior reconstruction, no subagent.

## Baseline / repaired paths

Same tree D:/autostack/.wt/lang-822/auto-lang, plan-822-dev; reviewed commit5490d6678cadb6e71cc01a96474151b56f4754f4, repair base90074e87f6c073868a0ee1f0642501bf383b7846, original implementation base de93bacc0ad16622140b0adb4d36eee86fe7fc83. Bookkeeping e4adcc19ac91b69f0bb0ca4c5cd340557ba54d25 verified (execution_done7/9). Entry/final HEAD match; tracked implementation clean. Existing untracked R3/R8 probes preserved; R13 temporary probes saved here then removed.

Cargo.lock SHA25637C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873. Main Plan/contract/Spec input hashes in input-hashes.txt, frozen SD sd-frozen.md SHA2564FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739. Dependencies/wire/verifier unchanged. No canonical Spec or ledger publication.

R12 normal/frame/replaced-value paths are repaired: min_buffer_id, drop_owned_reverse and deep_size are explicit worklists; commit_slot_write and list element replacement deaths route into explicit cleanup. Rc uniqueness checks preserve shared responsibility transfers; final unique Record/Variant payloads are unwrapped and queued. Independent original 64-layer control succeeds0/live0, steps8832 (replay-r12.log.txt). Work's four new deep-cleanup controls independently executed, including3072-level/256KiB normal cleanup,300-level trap and switch replacement. cleanup.log.txt and deep-chain.log.txt overlap two cases: count four UNIQUE committed controls, not six. Reverse cleanup3/3 passes; move_ filter5/5 passes (includes remove/fresh-place controls). reverse.log.txt/transfers.log.txt. Changed committed Rust files pass rustfmt --check --edition2021 and git diff --check90074e87f HEAD; targeted builds warning-free.

These close the original recursive helper examples within the paths tested, but the R12 whole-cleanup invariant is PARTIAL because a required clone rollback exit still uses host drop glue.

## F-822-R13-01 — P1: partial clone rollback bypasses explicit cleanup and overflows the stack

Classification: residual R12 last-owner/failure-exit cleanup gap, not a new scale promise; AC-03/05, T-03/T-04/T-06, V-03/06/11, SD-01/02. Existing Plan §5.1.6 requires explicit graph cleanup; §5.2/§5.4 and prior R8–R12 closure require partial clone cleanup on resource/cancel/fault exits, independent of normal fuel exhaustion.

Established code: clone_owned_value builds fresh leaf-first output into `out: Vec<Option<Value>>`. Already assembled nested Variant payloads are independent Rc chains. Its charge!/poll return Err directly, so a mid-assembly abort automatically drops out and any in-scope temporary assembled elements through Rust's ordinary Value/Variant/Rc destructors. This is NOT routed into the new drop_owned_reverse worklist. The ordinary dispatcher can clean source frames only after the helper returns; it cannot prevent helper-local teardown from overflowing first. Existing small List<Str> fault/budget tests do not exercise this nested assembled-output state.

Durable independent reproduction: partial-clone-replay.rs::review_r13_partial_clone_cleanup_small_stack uses the SAME3072-layer graph and256KiB stack as work's accepted nonrecursion witness. It obtains the type table through deep_variant_chain_module, passes real MIR and ABC admission, then builds a detached source value mirroring those distinct admitted Variant types and calls the ACTUAL production clone_owned_value seam. Fuel5572: planning needs3073 node units, then the leaf's structural+4byte charge and about2494 Variant assembly units occur before InstrSteps is due. Expected controlled InstrSteps with cloned partial leaf freed; actual test process aborts STATUS_STACK_OVERFLOW(0xc00000fd/-1073741571) inside the helper before its return marker. partial-clone.log.txt.

The source is deliberately pinned (mem::forget after the helper call; unreachable on failing run) to exclude its caller-side final Drop. No source cleanup, formatting of a deep success value or final cloned-root Drop occurs before the failure. The last printed marker is 'before production clone'; the expected 'after production clone' line never appears. This is a direct production-helper rollback test, not a full-dispatch crash claim. Detached runtime source shape mirrors an admitted legal acyclic type table; no cycle or invalid TypeIdx is used. The test-specific pin isolates the failure and is NOT a proposed product workaround.

Independent controls in partial-clone-replay.rs::review_r13_partial_clone_controls all pass on256KiB:

| depth / budget | Result | Meaning |
|---|---|---|
|64 /100|InstrSteps, fuel101,live0|small partial assembly cleans|
|3072 /100|InstrSteps, fuel101,live0|abort during planning, before fresh chain assembly, is safe|
|3072 /unlimited|success,fuel6150,live1|same graph fully traverses; successful result deliberately pinned to exclude caller-side final drop|

partial-clone-controls.log.txt. Source and full successful result pins are scoped test isolation, released by process termination; no leak result is claimed for these controls. Their purpose is to isolate mid-assembly rollback from source preparation, planning and successful helper return.

Minimal closing scope: make all helper-local partial-output owners and assembled temporaries drain through explicit last-owner cleanup on fuel, cancellation, deadline and allocator failure exits; preserve original source and original resource outcome, reserved cleanup accounting and reverse-event semantics. Audit List/Record/Variant assembly temporaries as well as out; do not patch only this one threshold or forbid legal nesting. Retain normal/source-frame/replacement worklists and shared-transfer behavior. Replay failing same-shape case and the three controls, plus representative nested-owned fault/cancel cleanup and existing isolation/reverse-transfer controls. No larger depth contract or global regression is attached to this finding.

## Reproduction commands

Copy partial-clone-replay.rs to the same implementation tree as probe/tests/review_822_r13_partial.rs. Run in separate cargo invocations because the failure aborts its test process:

    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r13_partial review_r13_partial_clone_cleanup_small_stack -- --nocapture
    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r13_partial review_r13_partial_clone_controls -- --nocapture

The first currently exits STATUS_STACK_OVERFLOW; the second passes. Inherited tests in the copied source are not selected/count inflation.

## Acceptance / knowledge / routing

AC-01 prior first-case/oracle evidence retained; AC-02 repaired direct cleanup and R11 clone charging controls retain bounded pass evidence. AC-03/05 fail on actual partial-clone rollback abort. AC-04 prior admission evidence retained, verifier unchanged. AC-06/09 prior cache/recipe/compatibility/incremental evidence retains its recorded boundary. AC-07 fail; AC-08 pending merge. Remaining full AC/V/SD audit remains explicitly incomplete, not inferred from green counts.

SD-01/02 cannot claim complete nonrecursive cleanup/all clone failure exits. SD-03..06 retain prior disposition; metadata targets/touched_goals=[] scope rationale unchanged. No canonical publication. Omission scan identifies helper-local partial owners excluded from the otherwise improved explicit cleanup routing; this is in-contract required work, not approved deferrable debt.

Next: same-tree work, reopen T-03/T-04/T-06, executing4/9; preserve other tasks/history and repaired R12-path evidence. Commit targeted repair and re-review affected closure/remaining acceptance. Pending merge-owned gates remain frozen bare cargo t + cargo tv + independent experimental workspace finite-contract checks and actual additional touched tiers, only after final pass; cargo tf main-checkout batch-only. No implementation fix, merge/archive or worktree cleanup performed, no full tier/acceptance aggregate run on blocked code.
