# PLAN-786 independent final review R4 — needs_fix

Date: 2026-10-16 (Asia/Shanghai). Reviewer: Codex in this independent review chat; no implementation edits or delegated agents. Scope: final. Plan revision: 1.

## Frozen baseline

- Worktree: D:/autostack/.wt/lang-786/auto-lang; branch plan-786-dev.
- Reviewed commit: 079b3b8d412ed7e7c1abdaade66c732f7281a454.
- Base: a1d5ed3114b163fa8a10c3d8985f5ae664bdad2c; repair base: 322719893262352ae1f25d894a833571c044e434.
- auto-project-model tree: b4a5de79bf608a897944fb794f16201c850e3d29.
- Worktree Cargo.lock SHA256: DAA795C5B9A39810836A5B8770D939B9175D8C506CA595415B57EC22CD96BC00. Main checkout lock differs; all tests use the Plan worktree lock.
- scanner.rs SHA256: 9B55A0AF1F2B7476B0FD949B86F1AC97D8E875526813AEAE7BB2D797514229C2.
- Canonical Specs latest commit: ec934b0006acb9b9838def82d0fcaf438ae0153f. auto-man/project.md SHA256 C18FFAAFD22FB30244E9886B661F56C97BEED87EAD9CB3670C660B4914B195A4; project-model/project.md SHA256 C9CACD0D58A1FC5B6D023365D322A02D995A0282DDDB2B63A6BF1B6FD407C133.
- [Frozen SD](spec-delta.md) SHA256 03A9A31EC65846BC2049693EFBE121E02FE08B4DF0954E7B9639A529031C3A1C. Not approved for publication.
- Implementation clean at entry/final check, HEAD unchanged. Restricted-shell worktree access failed initially; approved elevated checks with explicit worktree path succeeded. No repo configuration changes. Process-local RUSTC_WRAPPER='' bypasses the previously observed sccache issue.

## Findings

### F786-R4-01 — P1 — quoted comment markers hide real recursion (AC02, T02/T04/T05)

`project_display_reader.rs:108-133` recognizes comment openers without string context. `extra: "/*"` is a complete string to the model, but the guard treats the remainder of the file as an unterminated block comment. A subsequent `deep:` 512-layer array bypasses the guard and aborts the process. The sibling forms `extra: ["//", <512 nested arrays>]` and `extra: ["#", <512 nested arrays>]` hide structural brackets on the same physical line and also abort.

Expected: frozen legacy name=n/title=ok returned safely (compatibility fallback permitted). Actual: Windows STATUS_STACK_OVERFLOW / -1073741571 (0xC00000FD) for all three forms. quoted-block 32/128 and plain512 are accepted controls. [Exact inputs and runner](replay.ps1), [raw output](counterexamples.log).

The readiness argument that comments never contain structural brackets is false when the comment marker is inside a string. Repair the conservative preflight against actual parser states without changing legacy outputs or introducing unapproved syntax restrictions. Permanently retain these cases and controls, with process isolation and external deadline.

### F786-R4-02 — P1 — string closing brackets cancel actual nesting (AC02, T02/T04/T05)

`project_display_reader.rs:137` decrements depth for every closing bracket, including strings. The fixture nests 512 arrays, each beginning with a string whose value is `]`, followed by a comma and the next array; its innermost value is 0. Every real array opener is immediately cancelled in the guard by the string closing bracket. The guard maximum is 1, while the actual parser must recurse through 512 arrays. Use [counterexamples.rs](counterexamples.rs) / [cancel-depth-512.at](cancel-depth-512.at) for exact bytes.

Expected: same legacy values without abort. Actual: 0xC00000FD. 32/128 same-family controls exit0 Model(Value). This case contains no comment marker, so fixing F786-R4-01 alone cannot close it.

Counting a superset of bracket *characters* does not prove an upper bound on structural *nesting*: added closing brackets can reduce the measured maximum. Repair the upper-bound argument itself and permanently test cancellation at recursive levels (array/object siblings as applicable), preserving the approved compatibility contract. No new requirement for arbitrary-scale parsing is introduced: these reuse the previously accepted 512-depth safety examples and string syntax.

## Coverage and closure

| Prior item | R4 result |
|---|---|
| R3-01 original examples | 8/8 standalone R3 inputs exit0; permanent R3 file replay passes. Full safety invariant remains OPEN via R4-01/02. |
| R3-02 new test warnings | CLOSED: lib-test output has no display-reader warning; fields are used in diagnostics/assertions. Standalone R3 probe's unused title_zh warning is probe-only, not the production/lib-test build. |
| R2-02 lifecycle | CLOSED retained: 30s hang vs 2s kill/reap control passes in reader9/9; ChildGuard paths inspected. |
| R1-02/03 | CLOSED retained: consumer11/11, touched formatting verified. |

| Acceptance | Verdict / evidence |
|---|---|
| AC01 | pass: real Vue call reads once; selected field loops removed; unique legacy compatibility scanner, DTO use covered by corpus/path tests. |
| AC02 | fail: four independent mixed inputs cause stack abort despite old field outputs being available. |
| AC03 | pass: locale/name fallback wiring and nonselected readers unchanged; no added reverse dependency; explicit-locale checks pass. |
| AC04 | fail: independent review completed but safety requirements remain unmet; SD safety wording cannot be approved. |
| AC05 | pending merge-owned, not executed. |

Reproduced on reviewed tree:

- `cargo test -p auto-man --lib display_reader --locked`: 9/9, [reader.log](reader.log), includes timeout control.
- `cargo test -p auto-man --lib project_model --locked`: 11/11, [adapter.log](adapter.log).
- `cargo test -p auto-project-model --locked`: 30/30, [model.log](model.log).
- Existing title/locale filters: each 1/1, [title.log](title.log), [locale.log](locale.log).
- `cargo check -p auto-lang -p auto-man --locked`: exit0, [check.log](check.log); pre-existing warnings remain, no new display-reader warning in check or lib-test output.
- rustfmt --check on both new reader files: exit0; git diff --check base..HEAD: exit0. Prior touched Vue formatting closure retained; no Vue change in R3 repair.
- R3 standalone replay: exit0, 8/8 [r3-replay.log](r3-replay.log).
- R4 standalone replay: 9 invocations, 5 exit0 controls and 4 stack aborts; overall runner exit1. Every child has 10s wait then kill/wait; none timed out. [counterexamples.log](counterexamples.log).

No whole-tier t/tv/tt/tb/tu/taa was run. No implementation repair, merge, canonical Spec or ledger publication, or cleanup.

## Routing

Return to same worktree under work. Reopen T02..T05, keep T01 and unaffected wiring/locale evidence. status=executing, current_step=1/7. Before another execution_done handoff, correct the upper-bound proof and coverage matrix: explain both quoted comment hiding and bracket cancellation, replay historical plus new controls with process isolation, and commit repairs. Recording these findings as debt does not approve deferral.

Merge-owned gates stay pending: final-pass frozen bare cargo t, SD/ledger deposition, merge/archive, batch regression due check and guarded cleanup. No UI generator/VM/aavm change in this scope, so no tu/tv/taa trigger.

