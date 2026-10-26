# PLAN-822 final repair review R4 — needs_fix

Date: 2026-10-26. Plan revision 5. Reviewer: current chat Codex, did not implement the repair; independently inspected committed changes and exercised real admission/update paths. No delegated reviewer.

## Frozen baseline

- Worktree: D:/autostack/.wt/lang-822/auto-lang; branch plan-822-dev.
- Reviewed commit: 7239e8f4774bc4f6ea7cc6354bd78f99f780b55a.
- Repair base: bf9e92cb472495037ce73daab5b0c1d2a4f09f7d; full diff base: de93bacc0ad16622140b0adb4d36eee86fe7fc83.
- Main bookkeeping checkout HEAD: 4c11f26a229ff2060d40d2983c5175831be01481. Plan read from main, implementation from its existing tree.
- Experimental Cargo.lock SHA256: 37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873, unchanged from R3. Canonical Spec inputs and contract/SD hashes remain identical to R3 (input-hashes.txt); Plan hash records pre-R4 bookkeeping. Exact SD frozen in sd-frozen.md, SHA256 4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739.
- Tracked implementation clean at entry and final check; HEAD unchanged. Pre-existing untracked review_822_r3.rs retained untouched. This review's temporary review_822_r4.rs was saved as replay.rs and removed after execution. No implementation fix or canonical/ledger write.

## R3 repair closure / independently run checks

The work handoff now supplies actual diagnoses and coverage adjustment: Place lacked a referent type, and releasing a cloned list handle was mistaken for final buffer destruction. Both diagnoses match the diff. The new Place(ListTypeRef) is carried through wire encode/decode and snapshot type remapping; get/set check the bound type. The destructor drains a list only for its last Rc handle.

- semantics.log.txt: 28 passed / 0 failed / 0 ignored. Independently inspected assertions, including wrong get/set and wrong mint rejection, legal/stale/retired Place controls, move length/content, owned-element lifecycle, shared-copy source drop, call move-in, and normal/trap/nested reverse cleanup. This closes the concrete R3-01 bool/i32 corruption and R3-02 moved-list-empty examples and their tested transfer siblings. It does not establish every new referent invariant: F-822-R4-01 remains below.
- session.log.txt: 2 passed / 0 ignored, link rejection and execution-trap candidates preserve the current session. Original R2-01 closure retained.
- New independent counterexample: replay.log.txt, 1 failed, 6 unrelated included tests filtered. Valid referent control succeeds with value=7, live=0, allocs=frees=1. Out-of-range referent is admitted twice and panics in the production consumer.
- git diff --check repair-base..HEAD clean. Committed targeted tests compiled without warnings. An attempted broad cargo fmt --all --check expanded to path-dependent root crates and reported existing formatting drift in untouched crates/a2r-*; it is not a clean scoped formatting receipt or a new Plan-owned finding. No files changed by that check.
- A preliminary incremental filter selected zero tests; it is not counted as verification. Corrected preserves_session filter executed the two named tests above. Initial probe invocation used the wrong apply_revision API shape, corrected before behavioral evidence was collected.

## F-822-R4-01 — P1: Place referent bypasses type-table validation and panics after admission

Existing rule: Plan §5.1.1/5, §5.5, AC-04 and V-01/11 require untrusted typed input to be rejected at its admission gate; panic is not a valid negative outcome. The new referent is a module TypeIdx, just as a List element is. An unused entry remains part of the untrusted module type table.

Locations: core/src/mir_verify.rs around 380 validates List.elem and later Variant payloads but has no corresponding Place.elem validation. core/src/program_snapshot.rs:379–385 now directly evaluates map[l.elem as usize] for every Place in an admitted unit. ABC decoding reads the new elem word and relies on the MIR verifier; it does not close the gap.

Minimal input: existing legal beta_unit(7), whose body creates List<i32>[7] and returns its element, plus one unused type-table entry MirType::Place(ListTypeRef { elem: u32::MAX }). No forged checked credential, invalid host call, huge data, or Place runtime operation is needed.

Command in the implementation tree after copying durable replay.rs to probe/tests/review_822_r4.rs:

    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r4 review_r4 -- --nocapture

Expected: MIR/ABC admission returns a controlled invalid type-reference result before link/dispatch; the production consumer returns Err without panic. Legal Place<i32> entry remains accepted and returns 7.

Observed: verify_mir=Ok; encode succeeds; verify_abc_bytes=Ok(LoadedModule); CollectionsUpdateSession::apply_revision produces/adopts a FunctionCode and reaches ProgramSnapshot::link, which panics at program_snapshot.rs:380: index out of bounds, len=4, index=4294967295. catch_unwind in the test records the panic; the production consumer itself does not catch it. The regression assertion then fails because MIR admitted the bad input. Valid control succeeds through the identical path.

Established cause: newly added Place referent is not checked during full type-table validation, while the newly changed type merger assumes every referent was validated. This is a repair regression / incomplete integration of the new typed representation, not an unsupported runtime feature request.

Required closure: enforce the referent validity/domain invariant over every type-table entry before issuing a checked credential (or explicitly reject unsupported shapes). Exercise MIR and actual ABC admission with malformed referents, including unused entries, and the real update/link consumer. Keep valid typed mint/get/set, transfer and stale controls green. Representative sibling inputs should include boundary index, self/cyclic or unsupported referent, and valid intern/remapping controls under the declared type semantics. Reject deterministically without panic; do not merely bounds-check one indexing site while continuing to certify malformed types. This does not require a global suite or another memory/performance campaign.

## Acceptance and Spec boundary

| AC | R4 disposition |
|---|---|
| AC-01 | Existing first-case evidence retained; malformed typed-input integrity still fails under AC-04, no whole AC pass inferred. |
| AC-02/03 | Concrete R3 corruption and shared transfer defects closed on inspected/tested siblings; full new type representation cannot be certified with F-R4-01 open. |
| AC-04 | fail: real malformed type table reaches admitted credential and host panic. |
| AC-05/06 | Prior resource/cache/compatibility evidence retained within its previous boundary. Full remaining acceptance audit pending; no unrelated global rerun. |
| AC-09 | Prior failed-candidate closure retained, two independent tests pass; new malformed type path through consumer fails. Unchanged scalar performance campaign not replayed. |
| AC-07 | fail: F-R4-01 open; remaining complete AC/V/SD audit cannot grant pass. |
| AC-08 | pending merge-owned closeout. |

SD-01/02 typed Place/admission claims remain unapproved; SD-06 cannot treat malformed candidates as safely refused by the current consumer. No canonical or ledger deposition. Existing exact impact paths and empty touched_goals rationale remain valid. Full SD and unexamined AC/V items from R3 stay pending rather than inheriting work's all-green total.

Next: executing 4/9; reopen T-03/T-04/T-06, retain completed T-01/T-02/T-05/T-09 and all historical evidence. Repair and commit in the same tree, then targeted re-review. Preserve the new typed referent's whole-table invariant and its consumer assumptions in the next diagnosis/coverage record.

Merge-owned frozen bare cargo t + applicable cargo tv + experimental workspace finite contract checks and any additional real surface tier remain pending, only after final acceptance/Spec review passes. No whole-tier tests, aggregate acceptance, merge/archive/cleanup were performed. cargo tf remains batch-only on main.
