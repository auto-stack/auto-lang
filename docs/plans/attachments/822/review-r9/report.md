# PLAN-822 final repair review R9 — needs_fix

2026-10-26; Plan revision 5; final review by current chat Codex, not the implementer. Independent artifact/code/behavior review, no subagent. Implementation paused; no implementation repair or merge.

## Bound baseline and targeted results

Existing tree D:/autostack/.wt/lang-822/auto-lang, branch plan-822-dev, reviewed HEAD 7402022b5976531b6c685104d87d7354111a0766; repair base 952c2b586a6e040fd1637cbbb983c633f1cb0b78; original implementation base de93bacc0ad16622140b0adb4d36eee86fe7fc83. Entry/final HEAD match, tracked implementation clean. Pre-existing untracked R3/R8 probes preserved. New temporary review_822_r9.rs saved as replay.rs then removed.

Cargo.lock SHA256 37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873. Repair changes av.rs, semantics tests and script required IDs, not dependencies, wire schema or verifier. Main-checkout input hashes bound in input-hashes.txt. Frozen proposed SD hash 4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739. No canonical Spec/ledger publication.

- Original independent R8 alias replay now passes: CLONE [7], mutate source to [9], clone reads 7, Success/steps131/live0. replay-r8.log.txt.
- Five committed clone controls independently run: 5 passed, 0 ignored. clone-controls.log.txt. Basic list isolation, variant payload cloning, fault unwind and slot-budget controls work in those inputs.
- Three nested Variant clone is a legal MIR/ABC-admitted success, disc0/live0. This contradicts the repair's claimed one-level recursion bound.
- One unused self-referential Variant is accepted by both real gates; rejection oracle fails. variant.log.txt: 1 valid control passed, 1 invalid-input test failed, exit101.
- Changed committed Rust files pass rustfmt --check --edition 2021; git diff --check 952c2b586 HEAD clean; targeted compilation emits no product warnings.

No whole tier, workspace suite, acceptance aggregate, unrelated performance or GC/scale campaign. Prior R7 memory/provenance evidence remains applicable because its implementation, dependencies, configuration and inputs are unchanged.

## R8 disposition / F-822-R9-01 (P1): recursive clone traversal and missing cancellation remain

R8 old-example closure is confirmed; the full closing invariant is only PARTIAL. New repair-mechanism finding F-822-R9-01 affects AC-02/03/05, T-03/T-04/T-06, V-03/11 and SD-01/02. Existing Plan §5.1.6 requires explicit graph traversal; §5.4 requires data-scaled clone with cancellation points. The R8 closing oracle explicitly required bounded explicit traversal and clone cancellation; neither is delivered by a byte-budget test.

Established code: clone_owned_value at av.rs:656 now calls itself for List elements (:740) and Variant payload (:751). Its signature has no CancelFlag, deadline, fuel or polling callback, and no poll exists within its planning/assembly loops. CLONE adds the whole clone cost to st.steps before invoking this helper; the dispatch loop checks cancellation/step/wall limits only after helper return, on the next outer iteration. Upfront byte/field charging is accounting, not a cancellation point inside the work.

The justification 'frozen element domain forbids nested lists, so recursion exactly one level' does not bound Variant nesting. contract-decision §2 permits nested Variant, and check_collection_elem_kind explicitly accepts MirType::Variant for Variant payloads. replay.rs::review_r9_acyclic_variant_clone_control constructs three distinct acyclic Variant layers through real VariantPack/Clone instructions, passes MIR/ABC, executes successfully. Each payload enters the recursive visitor again. This proves the claimed one-level bound false without demanding huge inputs or triggering process stack overflow. No stack crash or measured cancellation-latency failure is claimed; explicit host recursion and absent in-loop poll are directly established contract violations. They are a repair design gap, not a new scale promise.

The five new controls contain no cancellation test. The 'owned elements independent' test rewrites text but reads the unchanged integer field, so its result alone is not a text-isolation oracle; strengthen the target-field/buffer assertions while adding the required clone/failure controls. This is a bounded test-quality note, not a separate product finding.

Closing scope: replace supported List/Variant recursion with budgeted explicit traversal consistent with the existing Record visitor; connect actual in-loop fuel/wall/cancel checks and partial-result cleanup. Exercise legal nested Variant and List<Variant<owned>> controls, original isolation replay, cancellation during meaningful clone work and nth-allocation failure with source/destination responsibility and cleanup assertions. Preserve ordinary Copy/Move and reverse cleanup. No arbitrary language depth cap, no disabling originally approved Clone, no aggregate acceptance demanded during repair. Reconcile work's one-level claims and retain the now-valid original R8 closure evidence.

## F-822-R9-02 (P1): Variant by-value cycles bypass the type-graph admission gate

Classification: original collection verifier integration omission exposed by the clone traversal audit, not introduced by 7402022b5. Affects AC-04/05, T-03/T-04/T-06, V-01/04/11, SD-01/02. Plan §5.1.6 rejects owned cycles; frozen contract §2 and SD-02 explicitly require Variant acyclicity. Full module type-table admission applies even to unused entries (same boundary established in R4).

Bound input: replay.rs::review_r9_variant_self_cycle_rejected starts from existing legal beta_unit(7), whole-module collections profile, then appends one nominal Variant Recursive { End, Next(Self) }. No instance of this recursive type is constructed, no invalid TypeIdx, no size edge, no runtime guest side effect. Real verify_mir returns Ok(CheckedMir), and encode_abc→verify_abc_bytes also returns Ok. Expected: controlled type-cycle rejection from both gates before trusted credentials. Independent test fails exit101. A three-layer acyclic Variant control is accepted and executes clone correctly.

Established cause: mir_verify.rs::check_record_acyclic (:942 onward) advertises Variant edges and has a later match arm for them, but the preceding `let fields = match ...` accepts only Record; every Variant takes the `_` branch, is marked BLACK, popped and skipped. Thus the new Variant edge extraction is unreachable. This is not an out-of-range-index problem and not merely absent test coverage. No runtime stack crash is asserted from this probe.

Closing oracle: make the existing graph walker actually visit all supported by-value Variant payload edges under shared verification budgets. Both MIR and ABC reject direct self and indirect Variant cycles (including table order variations); legitimate acyclic nested variants remain accepted through real consumer/clone paths; preserve existing Record cycle controls. Inspect combination edges within the supported domain. Record fields containing Variant remain intentionally unsupported; do not add a new field capability to repair this. Reuse existing gates rather than adding an alternative validator.

## Acceptance, SD and routing

AC-01 prior first-case/oracle evidence retained, no fresh full pass inferred. AC-02/03 partial/fail: original deep-copy alias example fixed, explicit traversal/cancellation incomplete. AC-04 fail: by-value Variant cycle trusted. AC-05 partial/fail: prior R7 analysis-memory repair retained, clone traversal/cancellation and graph rejection not closed. AC-06/09 prior cache/recipe/compatibility/incremental evidence retained within recorded boundaries; no unrelated replay. AC-07 fail; AC-08 pending merge. Other uncompleted AC/V/SD audit remains explicitly pending rather than inferred from test totals.

SD-01/02 cannot claim complete bounded clone or acyclic Variant admission. SD-03..06 retain their prior review boundary; metadata targets unchanged, touched_goals=[] rationale remains finite experimental scope. Main contract-decision §4 no longer claims collection CLONE/MOVE/DROP are rejected; this wording correction is valid, but does not waive explicit traversal/cancellation. No canonical publication.

Omission/workaround scan: repair closed shallow sharing but assumed a false recursion bound, and original cycle walker added Variant edge code behind a Record-only filter. These are required current-contract fixes, not deferrable debt. Record both in KNOWN-DEBT-AND-RISKS, reopen T-03/T-04/T-06, state executing4/9, preserve other tasks/history. Same-tree work must diagnose and cover these invariant gaps before re-review.

Pending closeout gates: only after final code/acceptance/SD pass, merge runs frozen bare cargo t + cargo tv + experimental workspace finite-contract gates and any actual additional touched tier. cargo tf remains main-checkout batch-only. No full gate on known-blocked code, merge, archive or cleanup.

## Reproduction

Copy replay.rs into the existing implementation tree at experimental/ac-av-probe/probe/tests/review_822_r9.rs, then from that tree:

    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r9 review_r8_clone_list_is_independent -- --nocapture
    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r9 review_r9_ -- --nocapture
    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_collections_822_semantics clone_ -- --nocapture

Only these filters were run; inherited unrelated tests in replay.rs were not selected or counted.
