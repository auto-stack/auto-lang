# PLAN-786 final review R3 — needs_fix

2026-10-16; reviewer: Codex, this independent review context, not the implementer; no delegation. Review scope: final; plan_revision: 1.

- Worktree: D:/autostack/.wt/lang-786/auto-lang; branch: plan-786-dev.
- Reviewed commit: 322719893262352ae1f25d894a833571c044e434.
- Base: a1d5ed3114b163fa8a10c3d8985f5ae664bdad2c; repair base: db71263b5b9b7f9451c6e588fef31765accbba3d.
- Start/end implementation status clean, HEAD unchanged. Main's unrelated pending changes preserved.
- auto-project-model tree: b4a5de79bf608a897944fb794f16201c850e3d29; Cargo.lock SHA256 DAA795C5B9A39810836A5B8770D939B9175D8C506CA595415B57EC22CD96BC00; scanner.rs SHA256 9B55A0AF1F2B7476B0FD949B86F1AC97D8E875526813AEAE7BB2D797514229C2. Group auto-down@c39ac7f is the unchanged recorded resolution dependency, not a changed component.
- Canonical Spec versions/hashes: see spec-delta.md. No Spec/ledger publication or implementation modification.

## Readiness and prior findings

The required two-failure diagnosis/coverage adjustment table was supplied and inspected. It incorrectly treats wider string skipping as a conservative bracket bound: wider skipped regions can hide real parser input. In particular, skip_to_entry_end is linear locally but returns to the outer document parser, which may subsequently recurse. Therefore its local lack of recursion is insufficient to prove the whole read path safe.

| Finding | Closure on this baseline |
|---|---|
| F786-R1-01 / F786-R2-01 | Original plain and multiline-double-quote examples closed; full safety invariant OPEN, continued by F786-R3-01. |
| F786-R1-02 | CLOSED: committed fixtures retain direct tests but do not widen 780 corpus identity; project_model 11/11. |
| F786-R1-03 | CLOSED: same Vue changes as R2, no new Vue edits; reader files rustfmt clean. |
| F786-R2-02 | CLOSED for the reported lifecycle defect: ChildGuard owns the child; deadline and poll error terminate/reap, Drop handles early exits; existing deep probe plus deterministic 30s sleep/2s parent timeout pass in 2.13s. |

## F786-R3-01 — P1, string skipping still undercounts real nesting

AC02/AC04; T-02..T-05; continuation of the original compatibility/safe termination requirement, not a new scale contract. Location: project_display_reader.rs:93-119 (especially string skipping at 103-116); scanner.rs:838-874.

Minimal representative input (the exact 512-layer file is single-unclosed-512.at):

```text
name: "n"
title: "ok"
extra: 'abc
ignored: 0
deep: [[[ ... 512 nested arrays ... 0 ... ]]]
```

The guard regards the apostrophe as the start of a string lasting to EOF, skipping all deep brackets. Actual parse_value classifies a leading apostrophe as unsupported and calls skip_to_entry_end. That helper's string arm stops at LF; its final advance consumes that LF, scans ignored:0, then returns at the next LF. The outer parse_document subsequently reaches deep and recursively parses it. The claim that unterminated string input must first Err or be consumed linearly through the remainder is false for this path. The same route is reachable through f-string and call values.

Reproduction: run replay.ps1 in this directory. It compiles an independent executable importing the exact production reader and linking this worktree's existing auto-project-model rlib. Each input runs in its own process with a 10s external kill/wait deadline. Frozen old scan returns name=n/title=ok before any unrelated field matters.

| Case | Result |
|---|---|
| plain-512 | exit0, Compat(DeepNesting) |
| multiline-512 | exit0, Compat(DeepNesting), R2 original closed |
| single-unclosed-32 / 128 | exit0, Model(Value) |
| single-unclosed-512 | stack overflow, exit -1073741571 / 0xC00000FD |
| single-closed-later-512 | same abort (later apostrophe does not make the wider skip sound) |
| f-unclosed-512 | same abort |
| call-unclosed-512 | same abort |

Fix the admission bound against all actual parser reentry paths, including unsupported values and malformed strings; do not infer safety from a local linear skip. A bracket upper bound must not skip regions where any reachable parser state can count brackets. Preserve normal model-path controls and legacy values. Add permanent process-isolated controls for these inputs, replay R1/R2 originals, and update the diagnosis with the precise control flow and coverage argument. No implementation fix made during review.

## F786-R3-02 — P3, new test warnings violate health gate

AC04 / T-04/T-05; new repair regression. project_display_reader_tests.rs:997 has unused variable reaped_success; :888 and :892 both have unread diagnostics fields (derived Debug does not count for dead-code analysis). All three appear in adapter.log and are introduced by this repair. The work receipt's zero-new-warning claim only considered the non-test check; lib-test compilation exposes them. Handle the intended diagnostics/result use explicitly, or narrowly document an intentional diagnostic-only field; keep failure diagnostics and lifecycle assertions intact. These are separate from the existing unrelated repository warnings.

## Acceptance and targeted checks

| AC | Task/artifact | Verdict/evidence |
|---|---|---|
| AC01 | T01..03; reader/read_field, Vue production call | pass: actual call shares read result; original two Vue scans removed, DTO admission remains real. |
| AC02 | T01/02/04; frozen matrix, compatibility & termination | fail: standard 8 reader tests pass, but independent 4/8 probe cases abort. |
| AC03 | T03/04; Vue locale and original fallback; unselected files | pass: title/locale tests each1/1; same locale/name fallback code as R2; unselected implementation and dependency graph unchanged. |
| AC04 | T06; independent counterexamples, warnings, SD | fail: R3-01 and R3-02. |
| AC05 | T07 | pending merge-owned, not run. |

Commands run in the existing worktree, with RUSTC_WRAPPER='' scoped to the invoking process after sccache startup failed with Windows os10013:

- cargo test -p auto-man --lib display_reader --locked: 8/8, 2.13s (including kill/reap hang control). Console result preserved here; no repeat needed merely to capture raw output.
- cargo test -p auto-man --lib project_model --locked: 11/11; adapter.log.
- cargo test -p auto-man --lib pac_title_overrides_document_title --locked: 1/1; title.log.
- cargo test -p auto-man --lib pac_display_title_locale_chain --locked: 1/1; locale.log.
- cargo test -p auto-project-model --locked: 30/30 plus doc-tests0; model.log.
- cargo check -p auto-lang -p auto-man --locked: exit0; check.log. Existing warnings remain; new test-only warnings listed above.
- rustfmt --edition 2021 --check on both reader files: exit0; git diff --check a1d5ed311 HEAD: exit0.
- replay.ps1: exit1; counterexamples.log + exact .at files. Probe's unused title_zh warning comes from the standalone harness and is distinct from R3-02.

Unchanged code/dependency/configuration justifies retaining R2's broader original field mutation and nonselected-file evidence; this round reran affected reader/adapter/locale checks and added unsupported/malformed-value parser reentry combinations. This does not claim universal parser soundness. No full tiers executed.

Outcome: needs_fix. Retain T01, reopen T02..05, T06/T07 pending; executing/current_step1. Next: repair in the same tree, commit, provide full-invariant closure and diagnostics coverage, then affected final review. Frozen bare cargo t plus SD/ledger/archive/batch-due/guard-clean remain merge-owned pending after final pass.
