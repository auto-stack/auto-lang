# PLAN-786 final review R2 — needs_fix

2026-10-16; reviewer: Codex, same independent review chat as R1, not implementation context; no delegated reviewer/model identity claimed.
review_scope: final; plan_revision: 1
worktree_path: D:/autostack/.wt/lang-786/auto-lang
branch: plan-786-dev
reviewed_commit: db71263b5b9b7f9451c6e588fef31765accbba3d
base_commit: a1d5ed3114b163fa8a10c3d8985f5ae664bdad2c
repair_diff_base: bde9d0523198f1a15bc908c62a5b18b59324e61a
Dependencies unchanged: auto-project-model tree b4a5de79bf608a897944fb794f16201c850e3d29; Cargo.lock SHA256 DAA795C5B9A39810836A5B8770D939B9175D8C506CA595415B57EC22CD96BC00; scanner.rs SHA256 9B55A0AF1F2B7476B0FD949B86F1AC97D8E875526813AEAE7BB2D797514229C2. auto-down@c39ac7f remains the recorded group resolution dependency.
Entry/exit implementation tree clean, same HEAD. Main's unrelated concurrent bookkeeping preserved.
Spec source versions/hashes and proposed delta: spec-delta.md. No canonical Spec/ledger publication.

## Finding closure and new regression

| Finding | Result | Evidence and invariant |
|---|---|---|
| F786-R1-01 | old example closed / full invariant still OPEN | plain 512 array and 512 node cases now return Compat(DeepNesting). However multiline string + nested array still aborts. See F786-R2-01; this is the same input-safety requirement, not a new scale promise. |
| F786-R1-02 | CLOSED | Renamed all 8 fixtures to .at, keeping their direct reader assertions while excluding them from pac.at corpus identity. Adapter untouched; strict exception identity gate preserved. project_model 11/11 on committed tree. |
| F786-R1-03 | CLOSED within scoped requirement | Vue new name chain/locale assertion lines now follow R1 formatter suggestions. Both new reader files rustfmt --check pass; pre-existing whole Vue/lib drift does not justify broad reformatting. |

### F786-R2-01 — P1, incomplete safety guard (AC02, T02/03/04/05)

At project_display_reader.rs:103, max_bracket_depth exits quoted-string scanning at LF. Actual scanner take_bracket_extent and value-position scan_string keep scanning across LF until the closing quote. With this valid control shape:

```
name: "n"
title: "ok"
extra: ["hello
world", [[[ ... 512 inner arrays ... 0 ... ]]]]
```

The prefilter interprets the closing quote on the second line as a new opening quote and skips the remainder of that line, hiding all real nested brackets. Thus read reaches parse_document despite depth exceeding 64. Standalone probe includes the unmodified production reader and links the tree's actual auto-project-model rlib.

Observed: multiline-32 and multiline-128 exit0 / Model(Value); multiline-512 aborts with stack overflow, exit -1073741571 (0xC00000FD). Plain-512, escaped-newline-512 and nodes-512 controls return Compat(DeepNesting) / exit0. Exact files, counterexamples.rs, replay.ps1 and counterexamples.log are durable in this directory. Each child has a 10s external deadline and explicit kill/wait on timeout. Old oracle returns title=ok based on the unchanged title line.

Required repair: make the prefilter conservatively sound relative to parser lexical/string/comment handling, or use a different safe admission design. Verify normal LF and CRLF multiline strings, escape/quote transitions, structured arrays/objects and node nesting, malformed exits, plus admitted controls. Do not just special-case the one example or alter accepted display results. The 64 envelope may select compatibility; this does not authorize rejecting valid input or claim the current scanner grammar changed.

### F786-R2-02 — P2, new deadline test leaves child running (AC02/04, T04/05)

project_display_reader_tests.rs:909-914: the timeout arm only asserts the deadline. On expiry, panic drops std::process::Child without kill/wait. Therefore a hanging child (including a Windows crash-report wait) survives the parent test's deadline. This is a source-inspected failure path, not a claimed runtime hang reproduced in the passing normal child. The test also pipes stderr/stdout without consuming it, so failure output can be obscured.

Required repair: guarantee termination and reaping on every timeout/poll/error path, preserve useful child diagnostics, and exercise a deterministic timeout control proving owned child termination/reaping and nonzero failure. The deadline must protect the child lifecycle, not only the test caller. Use a scoped guard/helper if useful; avoid process-wide env mutation.

## Verification / acceptance

| Evidence | Actual result |
|---|---|
| cargo test -p auto-man --lib display_reader --locked | 8/8 pass, including old-example process probe (reader.log) |
| cargo test -p auto-man --lib project_model --locked | 11/11 pass, committed fixture identities (adapter.log) |
| cargo test -p auto-man --lib pac_title_overrides_document_title --locked | 1/1 pass (title.log) |
| cargo test -p auto-man --lib pac_display_title_locale_chain --locked | 1/1 pass (locale.log) |
| independent replay.ps1 | exit1: multiline-512 stack abort, five control cases exit0 (counterexamples.log) |
| cargo check -p auto-lang -p auto-man --locked | exit0 (check.log); existing unrelated warnings, no new reader warning |
| rustfmt --check --edition 2021 --config skip_children=true reader.rs reader_tests.rs | exit0 |
| git diff --check a1d5ed311 HEAD | exit0 |

R1 full 260-combination differential probe remains applicable to the unchanged field/legacy implementation. This round re-executes frozen V01/V02 tests and representative new guard combinations; earlier evidence does not cover new prefilter safety. Unchanged model crate's R1 30/30 evidence is reusable only for that crate's unchanged behavior; no claim it proves this new guard safe. No whole-tier t/tv/tt/tb/tu/taa/tf executed.

AC01 pass: one production entry and actual DTO values, two old loops gone, one compatibility scanner; AC02 fail: guard undercounts and deadline cleanup missing; AC03 pass: existing locale/fallback, selected scope and dependency direction unchanged; AC04 fail: cannot approve final acceptance/SD safety wording; AC05 pending merge-owned.

## Diagnosis required before next handoff

This is the second failed final review. Next execution_done must include a diagnosis/coverage adjustment table: why single-line bracket tests and a flat10k fixture missed multiline lexical disagreement, how prefilter lexical behavior is established against actual scanner behavior, and how child lifecycle failure exits are proven. Cover sibling combinations and accepted controls before another final review. Passing the original 512 case is not full closure of F786-R1-01.

Outcome needs_fix / status executing. Preserve T01, reopen T02/03/04/05 and their dependent acceptance evidence, current_step=1/7. Do not fix implementation during review. Retain successful R1-02/03 closure evidence. Freeze SD again after actual repair; explicitly register the new adapter in future migration ownership.

pending_closeout_gates: final acceptance pass, then merge-owned frozen bare cargo t; no ui_gen/book/VM/aavm scope, thus no added tu/tb/tv/taa. Spec/ledger deposit, archive, batch-due decision and guard-clean remain pending. No merge/cleanup/publication authorized by this verdict.
