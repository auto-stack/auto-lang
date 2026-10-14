# PLAN-771 independent final review R1 — needs_fix

2026-10-14; plan_revision: 2. Reviewer: current Codex review context, independent of the implementation conversation; no extra agent/model role claimed. Scope: final. Worktree: D:/autostack/.wt/lang-771/auto-lang, branch plan-771-dev. Reviewed HEAD ecd9530d0086d0bdae35ef09cf229f2fb6f91468; diff base 199d6509e60821c3c0a6414ecd0bedf484105e4d. Tree clean before and after review; no implementation edits. Main checkout HEAD 882f74f5573e59144ee20c6919d23f8f710ca3a6. Dependency code untouched; 772/774 archived, 770 not merged. Current canonical inputs hashed in spec-inputs.sha256; frozen SD in proposed-spec-delta.frozen.md. Metadata paths correct; touched_goals empty because this bounded experimental profile does not close a canonical goal (including GOAL-017).

## Independent reproduced gates

From the Plan worktree: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-record-771.ps1. All 18 steps passed, 10 cases; full ac-core tests 187/187, record_native 9/9, fmt and all-targets check zero warnings, old 741/745/767 gates passed. Durable gate-summary.txt / gate-context.txt include reviewed SHA/toolchain/product hashes. Root cargo t/tv/docs_gen prohibited by this scope and not run. No canonical Spec or ledger published.

## Findings (all blocking original contract gaps)

### F-771-R1-01 [P1] Copy elision aliases live input and breaks record value semantics

AC-02/04/05; T-04/05/06/07. native.rs:950 passes the assignment destination into RHS evaluation; :1463 reuses it as callee out slot; :870 forwards that slot into a returning constructor; :1180 forwards field destinations into nested field expressions. No disjointness check protects live RHS/parameter reads.

Two real PE counterexamples are attached: alias-ctor.at directly assigns `p = P { a: p.b, b: p.a }`; alias-sret.at assigns `p = swap(p)` where swap returns that constructor. Starting values (1,2) must become (2,1), exit 0. Both build successfully and execute exit **12** (second field became 2). Writing the first child into p.a corrupts the later p.a read. alias-control.at uses a fresh local q=swap(p), checks q=(2,1) and original p.a=1, and exits **0**. Thus the failing case is storage aliasing, not unsupported syntax or a mistaken oracle.

Repair: only forward storage when disjointness from all live input is proven; otherwise materialize RHS/result in fresh storage before committing assignment. Preserve source evaluation order and deep construction efficiency. Replay whole-local and nested-field assignment, direct ctor/record-returning call and live view/mut parameters, with an accepted fresh-destination control. Do not weaken supported aliasing assignment into a syntax rejection.

### F-771-R1-02 [P1] Positional record constructor missing

AC-02/05; T-03/05/06/07. source/resolve.rs:2124 resolve_call only looks up functions/intrinsics, never resolves a record name into positional construction. positional.at is a one-field Q declaration and `Q(7)`: check-source exits **1**, `source.undefined: no function named Q`, plus cascading field type error. Expected: accept and native exit 0. Plan §5.2 explicitly promises position and named construction; frozen SD-01 also claims `Pair(1,true)` support. Existing tests exercise only braces/static new.

Repair: implement declared-order positional construction with arity/type/source-order diagnostics, preserving legitimate user function and name conflict rules; source/Atom/HIR/PE controls needed. Update support assertions only after evidence.

### F-771-R1-03 [P1] Actual Bundle call binding mutation panics verifier

AC-03; T-02/06/07. verify.rs:901 directly indexes eval_args[b.arg] in record ownership checks, before the later binding bounds validation (:1678).

bundle-mutation.rs binds the real r01 fixture, verifies its unmodified clone succeeds, then sets a nonempty Call bindings[0].arg=u32::MAX and asserts mutation occurred. catch_unwind around the real verify_record reports **PANIC**, src/verify.rs:901 index out of bounds (len 2, index 4294967295). A sibling mutation of eval_args[0].0=u32::MAX is a controlled rejection, so the harness distinguishes the two. Expected: diagnostics Err, no Checked/backend or panic.

Replay harness without editing implementation:

    $rlib=Get-ChildItem experimental/ac-core/target/debug/deps/libauto_ac_prototype-*.rlib | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    rustc --edition 2021 D:/autostack/auto-lang/docs/plans/attachments/771/review-r1/bundle-mutation.rs --extern "auto_ac_prototype=$($rlib.FullName)" -L dependency=experimental/ac-core/target/debug/deps -L native=C:/Users/zhaop/.cargo/registry/src/mirrors.aliyun.com-0671735e7cc7f5e7/windows_x86_64_msvc-0.52.6/lib -o experimental/ac-core/target/review771-mutation.exe
    & experimental/ac-core/target/review771-mutation.exe experimental/ac-core/fixtures/record-hir/valid/r01-records.atom

Repair: safe bounds handling before new ownership traversal; cover arg/param indices at boundary and max, missing bindings and malformed reference combinations through the actual verifier. Do not rely on textual binder validity for public Bundle verification.

### F-771-R1-04 [P2] Direct-HIR copy failure has same exit code as success

AC-02/06; T-05/06/07. r01-records.atom:119/137 first failed copy check returns c1e=1; success returns c1c=1. record_native.rs:105 and gate script:512 accept 1. Thus V-02's original-object preservation assertion cannot distinguish failure.

hir-broken-copy-check.atom differs only by c1d=99 instead of 1, forcing the original-object check into k1. It builds and executes **1**, exactly the accepted success code. This is an actual mutated input/real PE, not a mock banner.

Repair: distinct success and each failure branch codes, and deliberate failure controls proving both test/gate failure propagation. Add direct-HIR nested-copy/aggregate-return/ordering cases as promised by V-02/V-07 rather than citing source PE coverage as the direct-HIR leg.

## Acceptance and knowledge delta

| AC | Result | Evidence / limit |
|---|---|---|
| AC-01 | pass | record_hir identity/dispatch and profile isolation/receipt gates |
| AC-02 | fail | nominal/nested and fresh copies pass; F01 incorrect self-assignment; F02 missing positional; F04 ineffective HIR copy oracle |
| AC-03 | fail | graph/reference/receiver existing tests pass; F03 actual binding mutation panic |
| AC-04 | fail | existing mixed args/mut/receiver tests pass; F01 out-return overlaps live input |
| AC-05 | partial | trace ba/aab and 1024 type/4096 field-path tests reproduced; F01 source-order reads corrupted, F02 syntax missing; malformed partial deep Drop not established by recorded V09 cases |
| AC-06 | fail | all advertised gates reproduced, but false-complete coverage/SD claims need correction; 770 second-landing combined baseline gate still conditional |
| AC-07 | fail | independent audit complete, four blocking findings |
| AC-08 | pending | merge-owned; no merge/archive/cleanup authorized or performed |

SD-01 falsely claims positional support; SD-02/03 trusted front gate and disjoint out-slot obligations not met; SD-04 V01..V10 all-green claim unsupported; SD-05 bounded ACC attribution valid only after supported syntax is proven. Frozen delta is review evidence, not approved canonical text. Current Spec targets read on main; no implementation-phase delta published. patch_a2.py/patch_r1.py are one-off committed implementation helpers; remove or explicitly justify in repair closeout inventory (nonblocking hygiene observation, not added scope).

## Replay CLI

In the same worktree use experimental/ac-core/target/debug/auto-ac-prototype.exe:

    check-source <attachment>/positional.at --source-profile auto.source.record.draft
    build-source <attachment>/alias-ctor.at --source-profile auto.source.record.draft --entry main --output experimental/ac-core/target/review771-alias-ctor.exe
    build-source <attachment>/alias-sret.at --source-profile auto.source.record.draft --entry main --output experimental/ac-core/target/review771-alias-sret.exe
    build-source <attachment>/alias-control.at --source-profile auto.source.record.draft --entry main --output experimental/ac-core/target/review771-control.exe
    build <attachment>/hir-broken-copy-check.atom --entry d_main --output experimental/ac-core/target/review771-hir-broken.exe

attachment=D:/autostack/auto-lang/docs/plans/attachments/771/review-r1. Run generated PEs and compare exact exit codes above. All review probes terminate immediately; all gate/test PE runs use existing contained executor.

Next: auto-plan:work in the existing tree, close F-771-R1-01..04 with regression assertions and committed repairs, update Case/SD claims, rerun required gates and hand back for independent final review. T-02..07 reopened, T-01 preserved: executing/current_step 1/9. No semantic Plan revision performed.

Final product identity: gate-context.txt records the binary present at gate entry, before its build step. The gate rebuilds the binary (normal operation); final-products.sha256 records the rebuilt binary actually used by the independent counterexamples: df219c89e857ce55dd3e6fe2d8a532abf73b13b0419ce05857d541dbf475686f. This is build-output drift, not source/HEAD drift; final implementation HEAD and clean status rechecked. The proposal's frozen/current SHA256 are included for delta binding.
