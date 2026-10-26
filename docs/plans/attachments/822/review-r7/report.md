# PLAN-822 final repair review R7 — needs_fix

2026-10-26; Plan revision 5; final review. Reviewer: current chat Codex, not the implementer of this repair, continuing independent artifact/code/behavior review. No subagent. This round closes the prior specific semantic findings but does not grant complete acceptance.

## Bound baseline

- Existing tree `D:/autostack/.wt/lang-822/auto-lang`, branch `plan-822-dev`, reviewed commit `3afbcbd3f74ed3d59e562dd901c419678f92f58e`.
- Repair base `352cbc5c4e1afe4dd1a515e856d4986503506a22`; full implementation base `de93bacc0ad16622140b0adb4d36eee86fe7fc83`.
- Main bookkeeping HEAD `cd85515ad5c7b975e89c51fccb0755933c405cce`; shared Plan entered execution_done 7/9. Plan contract revision remains 5.
- Cargo.lock SHA256 `37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873`, unchanged. Dependency manifests, wire/type schema and runtime executor are unchanged by this repair; existing dependency/window receipts remain applicable. `input-hashes.txt` binds current canonical Spec/contract/SD inputs, all matching R6 apart from Plan bookkeeping.
- Frozen SD `sd-frozen.md`, SHA256 `4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739`. No canonical Spec or ledger publication.
- Entry/final implementation HEAD match and tracked implementation is clean. Existing untracked R3 probe remains untouched, SHA256 `3F84BA196CFDDF2853D5102D38FEA83FF45DBC4E09955A65738A0F10988306D8`. This review's temporary R7 probe is saved as replay.rs and removed from the tree.

## Prior findings and targeted checks

The repair diagnosis matches the code: predecessor roots now merge independently of tokens_live, root equality observes retained provenance, and Copy/Call no longer activate a Place borrow. These changes repair the reproduced semantic failures.

- **F-822-R6-01 CLOSED**: original independent Jump escape is rejected by both real MIR/ABC gates; valid external-root Jump return and cross-block read return 7. Independent mixed local/external predecessor test rejects the local possibility in both predecessor orders; external-only diamond is accepted, executes to 7/live=0. This is a stronger join control than minting the same local token before both branches.
- **F-822-R6-02 CLOSED**: direct and copied retained Places both allow the original update, and committed externally rooted helper-roundtrip-then-update returns 9. Old stale/generation and typed read/write controls remain green. This also closes the remaining semantic escape/field-domain portion of F-822-R5-02 within the inspected supported domain; earlier unrelated closure evidence is retained.

| Evidence | Result / scope |
|---|---|
| replay-r6.log.txt | Original independent R6 filter: 5 passed, 0 ignored. |
| join.log.txt | One independent test, three subcases: mixed local/external roots in both orders reject; external-only diamond executes correctly. |
| place.log.txt | Committed `place_` filter: 16 passed, 0 ignored, including alias, frame, type, stale/fresh and escape controls. |
| roundtrip.log.txt | Committed helper-roundtrip/update: 1 passed, 0 ignored. |
| memory.log.txt | Small valid control passes; larger otherwise legal input admits at both gates and exceeds the frozen analysis-allocation cap. Test fails as required by the missing guard. |
| fmt.log.txt | Changed committed Rust files, rustfmt --check --edition 2021: exit 0. |
| git diff --check 352cbc5c4 HEAD | Clean. |

New memory probe is run singly with --test-threads=1. It audits the analysis-accounting omission explicitly identified in R6's closure scope; it is not a general memory/performance campaign or a new scale promise. No whole-tier cargo t/tv/tt/tb/tu/taa, whole-workspace suite or aggregate acceptance was run. Previously reported 821 ownership tests are retained as work evidence rather than presented as freshly executed here. Product builds now emit a real dead-code warning, detailed below.

## F-822-R7-01 — P1: Place root allocations bypass the frozen verifier memory guard

Classification: original Place integration omission, still present after the provenance repair; not a new feature request or a requirement to accept larger programs. Affects AC-04/05, T-03/T-04/T-06, V-07/11 and SD-01/02. Plan §5.2/§5.4 require type/path analysis to remain within shared budgets and reject resource overflow before the allocations. The inherited contract.rs MAX_VERIFY_ANALYSIS_BYTES=16MiB explicitly covers composite verifier allocations in staging. R6 closing scope already requested auditing analysis accounting when integrating retained roots.

Bound input: `review_r7_place_provenance_memory_budget` in replay.rs, built from the existing beta_unit(7) fixture. One whole-module function initializes List<i32>[7], then mints 12,000 Place<i32> locals referencing its valid element, finally returns a fresh list read. It is acyclic, has no escapes, and does not require a huge runtime list. It has 12,004 slots, 12,005 instructions, one block, and a 216,192-byte encoded ABC, below existing input/table limits. No product cap is increased and no guest execution is needed to observe admission allocation.

Expected: charge retained Place root matrices conservatively and return ResourceKind::VerifyMemory before allocating analysis beyond 16,777,216 bytes; an allowed small control must remain accepted. It is valid to resource-reject this large input, not necessary to support its size.

Observed real gates: MIR=Ok and ABC=Ok; credential analysis_bytes reports only 2,340,928 bytes. Independently calling the exact production ownership::verify_owned_function used by the MIR verifier, after constructing the input, also returns Ok and charges 2,340,928 bytes while the integration test's System allocator records **21,217,630 bytes of incremental live-allocation peak**, exceeding the frozen 16MiB analysis limit. Construction, decoded-module storage, checked-module cloning and retained earlier credentials are outside that measurement window; the test resets the peak against the current allocation baseline immediately before this production call. A single test thread is used. The counter counts requested live allocation sizes, not worker private bytes or the separate 128MiB process policy. Realloc allocator-internal transient overhead is not claimed as measured.

Valid control: 64 Places, 68 slots/69 instructions, 1,344-byte wire, both real gates and direct ownership analysis accept; measured incremental analysis peak 19,614 bytes, charged 13,504 bytes, below the cap. These two sizes distinguish small legitimate input from a cap-exceeding allocation; the assertion is the frozen budget, not an invented threshold at exactly 12,000 Places.

Established cause: ownership.rs:252..262 classifies token_slots for the preallocation bound using only StrView/MutRef, omitting Place. The subsequent bound at :281..299 therefore budgets zero root bitsets for Place slots. LIST_PLACE creates a full Bitsets(n_slots+1) per retained reference. Even one simultaneously retained set of the 12,000 root bitsets alone requires `12000 * ceil(12005/64) * 8 = 18,048,000` bytes, already beyond 16MiB before state/header overhead; this lower bound independently substantiates the measured breach. The new join/equality rules retain that provenance but do not repair the budget's classification.

Closure oracle: account for all supported provenance-bearing kinds and live analysis-state copies before allocation, retaining shared cap semantics. Replay small legitimate Place input, the oversized Place input through MIR and ABC, and a representative cross-block/alias retained-root shape; assert controlled VerifyMemory rejection before the forbidden allocation and continued semantic controls. Reuse existing allocator/analysis infrastructure where suitable. Do not fix by dropping roots, weakening escape checking, raising the budget, changing the expected outcome to late allocation failure, or demanding full performance/GC suites. A global regression is not attached to this finding.

## F-822-R7-02 — P2: repair leaves a product dead-code compiler warning

Classification: repair hygiene regression, AC-07/T-06 (and repository mandatory review Health Check). The repository requires zero unhandled compiler warnings. After the repair removes the predecessor-merge call, OwnState::is_token_live at ownership.rs:1602 is unused. Every targeted build emits `warning: method is_token_live is never used`, from committed auto-av-core rather than the pre-existing untracked R3 probe. The work handoff's “零 unused warning” claim is contradicted by raw logs.

Expected: no unhandled product warning; observed auto-av-core generates 1 warning. Evidence is replay-r6.log.txt, join.log.txt, memory.log.txt and roundtrip.log.txt. Closure: remove or meaningfully use the obsolete helper and recheck the affected core build/targeted tests for warnings, without blanket lint suppression. No whole-tier run is required merely to resolve this warning.

## Acceptance / SD / routing

| AC | Disposition and evidence boundary |
|---|---|
| AC-01 | Prior first-case and parent-oracle evidence retained, no complete fresh pass inferred. |
| AC-02/03 | R5/R6 Place semantic findings closed within inspected paths; original move/cleanup evidence retained. Full remaining AC/V audit still pending rather than implied by counts. |
| AC-04/05 | fail: trusted admission permits analysis allocation beyond the frozen cap. Separate runtime rollback/peak evidence is retained and does not discharge this verifier staging requirement. |
| AC-06/09 | Prior cache/recipe/compatibility/incremental evidence retained within recorded boundaries; no unrelated replay. Remaining full audit pending. |
| AC-07 | fail: two current findings, including an unhandled product warning. |
| AC-08 | pending merge. |

V-07/11 resource accounting is affected; semantic V-01/02/12 closure evidence improves, with other required cases retaining their prior bounded disposition. The remaining final AC/V/SD audit is not claimed complete after the reproduced work-owned blocker. No global run was spent on known-blocked code.

SD-01/02 cannot claim fully budgeted retained Place analysis yet. Their semantic lifetime/alias descriptions can retain the verified closure above. SD-03..06 and exact target/impact metadata retain prior review boundaries. The unchanged SD-02 guard `must-intersect join` wording still needs reconciliation with the previously recorded conservative/block-local gate boundary before deposition; no canonical change or semantic weakening is inferred. touched_goals=[] remains justified by the limited experimental scope. Hashes and frozen proposed text are in this directory; no live Spec/ledger publication.

Omission/workaround scan: the new provenance-bearing type was integrated into semantic classification but not the preallocation budget; work claimed complete integration and warning-free builds without covering those consumers. These are current-contract failures, not approved deferrable debt. Return to same-tree work, reopen T-03/T-04/T-06, retain T-01/T-02/T-05/T-09, state executing 4/9. Commit fixes and re-review only affected findings/controls plus remaining acceptance audit as appropriate.

Pending merge-owned gates: frozen bare cargo t + cargo tv + experimental workspace complete finite-contract checks and any actual additional surface tier, only after final acceptance/Spec pass. cargo tf remains main-checkout batch-only. No implementation repair, merge, archive or worktree cleanup occurred.

## Reproduction commands

Copy replay.rs to the existing tree as experimental/ac-av-probe/probe/tests/review_822_r7.rs, then from that tree run:

```text
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r7 review_r6 -- --nocapture
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r7 review_r7_mixed -- --nocapture
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r7 review_r7_place_provenance_memory_budget -- --nocapture --test-threads=1
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_collections_822_semantics place_ -- --nocapture
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_collections_822_semantics helper_roundtrip -- --nocapture
```

replay.rs includes inherited earlier helpers/tests; only the named filters ran this round. Original R6 replay ran before adding allocator/join probes to the temporary file; original inputs and assertions were unchanged. No inherited full suite was run or double-counted.
