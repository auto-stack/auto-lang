# PLAN-786 independent final review R5 — pass

2026-10-16 (Asia/Shanghai). review_scope=final; reviewer=Codex in this independent review chat, not the implementation context; no delegation or implementation edits. plan_revision=1.

## Baseline

- worktree D:/autostack/.wt/lang-786/auto-lang; branch plan-786-dev.
- reviewed_commit bb3cd34b78610ddbdc39d50e682eec7174896892.
- base_commit a1d5ed3114b163fa8a10c3d8985f5ae664bdad2c; repair_diff_base 079b3b8d412ed7e7c1abdaade66c732f7281a454.
- auto-project-model tree b4a5de79bf608a897944fb794f16201c850e3d29 (unchanged).
- Worktree Cargo.lock SHA256 DAA795C5B9A39810836A5B8770D939B9175D8C506CA595415B57EC22CD96BC00; scanner.rs SHA256 9B55A0AF1F2B7476B0FD949B86F1AC97D8E875526813AEAE7BB2D797514229C2 (unchanged).
- Both canonical Spec inputs remain ec934b0006acb9b9838def82d0fcaf438ae0153f, hashes in the [frozen SD](spec-delta.md). SD SHA256 69D8D31BA80D43B86B84A93AADE32095B79EBA6EC91D937BCF028E1E83C4DA22.
- Implementation clean at entry and final check, HEAD unchanged. Tests explicitly use the Plan worktree, not the main checkout's different lock. Process-local RUSTC_WRAPPER='' retained to bypass the previous sccache environment issue; no config edits.

## Closure / independent reasoning

| Finding | Result |
|---|---|
| F786-R4-01 quoted comment markers | CLOSED. Original quoted-block/line/hash512 each exit0 DeepNesting on this commit. No comment recognition remains in the preflight. |
| F786-R4-02 string closing brackets | CLOSED. Original cancel32 Model, cancel128/512 DeepNesting, all exit0. No decrement remains in the preflight. |
| R1-01 / R2-01 / R3-01 safety invariant | CLOSED within the reviewed consumer envelope. Original R3 8 cases and R4 9 cases all pass independently, plus permanent process probes/fixtures and new boundary combinations. |
| R3-02 test warnings | CLOSED retained: lib-test build logs contain no new reader warning. |
| R2-02 child lifetime | CLOSED retained: unchanged ChildGuard/run_child_probe reviewed in R4; rerun deterministic 30s hang/2s deadline check passes as part of reader9/9. |
| R1-02 corpus / R1-03 formatting | CLOSED retained: consumer11/11, touched-file rustfmt/diff-check pass; no further Vue edits in the repair. |

The revised preflight is a literal count of '[' and '{', not a lexical depth calculator. Every structural nesting level in the document/value scanner consumes such an opener; sibling entries/elements are looped. Node bodies, structured arrays/objects, descriptor assembly and value destruction have nesting bounded by this source structure. Calls in structured value positions are rejected by scan_value_positions before AtomParser; document argument extent collection is iterative. Therefore the previously dangerous strings/comments/closing-token combinations cannot reduce the count. The tests separately check node/object/mixed nesting and a long argument-list control, rather than assuming array evidence covers them.

This is deliberately conservative: flat groups or brackets in strings/comments can trigger compatibility. That changes the internal audit route, not the old selected field outputs. This is compatible with the approved fallback contract and does not impose a syntax rejection. The 64 threshold is the established empirical consumer margin; this review does not assert unlimited resources or arbitrary small thread stacks.

The work-generated quoted-marker helper contains a malformed deep-partial token in some line-marker forms; its >64 fallback assertions alone would not prove the original parser path. The independent R4 runner and byte-preserved fixture replay use the original valid line/hash forms, and both pass. No acceptance evidence relies on that helper alone.

## Acceptance / coverage

| ID | Tasks | Result / evidence |
|---|---|---|
| AC01 | T01/T02/T03/T05 | pass. Actual Vue generation reads the three fields once via DisplayRead; old scalar/name loops deleted, one legacy scanner remains. Corpus/path assertions show DTO use remains real (work receipt 170 files/510 reads, model496/compat14; independently rerun corpus assertions pass). |
| AC02 | T01/T02/T04/T05 | pass. Frozen edge matrix, dynamic/bad input checks, permanent old fixtures, process isolation plus independent original and new sibling/boundary replays pass. No VM/file/env effects in production reader. |
| AC03 | T01/T03/T04/T05 | pass. Locale and fallback logic retained, both original title/locale tests pass. Nonselected source controls pass. No dependency manifest/reverse edge change. |
| AC04 | T06 | pass. Complete repair diff, unchanged call wiring/admission, scoped checks and SD reviewed independently; no open in-contract finding. |
| AC05 | T07 | pending merge-owned; not claimed complete. |

Reused R4 call-site/locale/dependency and dedup inspection because those sources/manifests and dependency tree are unchanged by bb3cd34b7. Re-read current read_field/legacy path and all changed preflight/test code; reran required acceptance tests against this commit. PLAN-784 t04 migration-inventory names the PLAN-786 display-reader and related Man migration surfaces; the frozen SD preserves the reader/compatibility scanner in that follow-up.

## Reproduced checks

All executed inside the same Plan worktree, locked dependencies; logs are durable under this directory.

- `cargo test -p auto-man --lib display_reader --locked`: 9/9, [display_reader.log](display_reader.log), including hang kill/reap control and 18 permanent R3/R4 byte fixtures.
- `cargo test -p auto-man --lib project_model --locked`: 11/11, [project_model.log](project_model.log).
- Existing title and locale filters: each 1/1, [title](pac_title_overrides_document_title.log), [locale](pac_display_title_locale_chain.log).
- `cargo test -p auto-project-model --locked`: 30/30, [model.log](model.log).
- `cargo check -p auto-lang -p auto-man --locked`: exit0, [check.log](check.log). Existing unrelated warnings remain; no new reader warning in check or lib-test output.
- rustfmt --check on both new reader files and git diff --check base..HEAD: exit0.
- Independent R3 runner: 8/8 exit0, [r3-replay.log](r3-replay.log).
- Independent R4 runner: 9/9 exit0, [r4-replay.log](r4-replay.log).
- New [R5 runner](replay.ps1) / [source](counterexamples.rs): 13/13 exit0, [counterexamples.log](counterexamples.log). Object/nodes/mixed/flat/quoted/comment at opener counts 64/65 verify Model(Value)/DeepNesting with name/title unchanged; args512 has only one opener and remains Model(Value). Every child protected by 10s external wait and kill/reap; no timeout occurred. This probe set is a representative boundary check, not an exhaustive new syntax requirement.

## Knowledge delta and routing

Both SD operations verified against unchanged canonical sources; exact revised wording is frozen in spec-delta.md. In particular, replace old zero-production-switch claims only for the selected Vue fields, and state the guard counts all opener characters (including flat/string/comment occurrences), not actual depth. Model syntax and other consumers remain as documented. No canonical Spec/ledger published during review.

Outcome final pass -> status reviewed, current_step 6/7, T06 complete. T07 remains open. Pending closeout gates: merge-owned bare cargo t on the frozen commit; both SD deposits/index/ledger; merge/archive; batch-regression due determination on main; mandatory worktree guard and cleanup. No ui_gen/VM/aavm/book changes, so no tu/tv/taa/tb trigger from this scope. No whole-tier regression was run during review, no merge/archive/cleanup performed, and delivery is not yet complete.
