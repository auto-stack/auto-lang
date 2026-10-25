# PLAN-848:r3 merge:r1 — blocked before landing

Frozen reviewed implementation: `6bc5abc6080b2261a43f55a54f127cd779239edb`.
Tree: `D:/autostack/.wt/lang-848/auto-lang`, branch `plan-848-dev`, clean.
SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`; original three locks unchanged.
Independent final review pass remains valid for this code. T10 is incomplete.

## Passed current gates

- Producer complete locked driver:216pass/0fail/0ignored; full bins build,
  all-targets check, formatting and source/SDK closure pass.
- Fresh external consumer, own681-file SDK and initially empty own target:
  complete216pass/0fail/0ignored, full bins build and source closure pass.
- Both source verifiers reject API drift, stale SDK pin, source-lock drift and
  missing project GLB, then accept restored inputs.
- Producer old two games positive/negative, original authoring positive/negative,
  interaction/wrong-target/drop-release/truncated controls and reopen pass.
  Hierarchy, actual transparent/ghost GPU sensitivity controls, all7asset and
  all7Play cases pass their original oracles.
- AGP163/build/16native plus independent counter AND content controls retain
  the23 explicitly identified delivered846 evidence slots. Entire AGP workspace
  source/tests/assets/config/manifest/lock is unchanged by Git diff and has no
  SDK dependency; this is evidence reuse, not a newly executed AGP campaign.

Actual commands and exit records: [gates.json.txt](gates.json.txt),
[producer-results.json.txt](producer-results.json.txt),
[baseline.json.txt](baseline.json.txt), [AGP reuse](agp-reuse.json.txt).

## Failure attribution

First consumer attempt omitted the existing native32MiB link-stack profile.
Its relocated asset child overflowed the default1MiB main stack. PE headers and
the original846 invocation establish the configuration discrepancy; same-source
exact-test replay passed with the original32MiB profile, then the complete216
consumer suite passed. The original failed run is preserved, not counted pass.
See [diagnosis](diagnosis.md) and [PE evidence](stack-diagnosis-before.json.txt).

The current blocking gate is producer user-source native verification. First
Patrol stopped at phase5; a same-binary/source/threshold Patrol-only control
passed phase27. The complete13-case retry passed Move/Patrol/Fast, then relocated
stopped at phase17 before its required V-hide result. Both failed raw reports
record `auto-Pause: tick backlog >4` and Paused input. The second failure is at
[user/relocated/report.json](user/relocated/report.json); the first is retained
under user-failed-attempt1 (original invocation directory user).

The observed blocking condition is real scheduler backlog invoking the existing
automatic pause, not a demonstrated segment numeric/credit regression. External
cause is unproven. Play::clock/dispatch_credit and the original v1 API/native
oracle are unchanged from846 (verified byte-equivalent function span). Other
Play changes are v3 constructor/profile/error handling; the inspected diff is
preserved in [blocked.json.txt](blocked.json.txt). We do not claim all Play code
is unchanged, or use old evidence to erase these new failures.

No product code, assertions, timeout, pause policy,128-node support,32credits or
CPU budget was changed. After a diagnostic and one complete retry, no further
automatic retry runs without a changed condition or new diagnostic evidence.

## Recovery checkpoints

prepared/landed/ledger_refreshed/archived/cleaned: **not executed**. Canonical
Spec hashes still match review inputs; autographics-dev remains bf22db346.
Plan stays reviewed9/10 with merge blocked; no implementation defect is yet
established that would justify reopening work. Remaining producer Step/projects/
spatial/segment matrix, all consumer native gates, both ownership-mutation
matrices and final raw audit remain pending. `audit.py` is a draft gate audit,
not executed and not evidence of a complete pass.

Next: establish a stable native replay condition or diagnose the backlog path,
then close the failed and pending original gates at the same thresholds. Reuse
the passed216 suites only after rechecking code/dependencies/configuration/input
identity. Any necessary code repair returns to work and affected independent
review before new frozen gates. Only all required gates passing permits landing,
canonical Specs/ledger, archive and guarded cleanup.

Both verified runner pairs are preserved under
`D:/autostack/plan-artifacts/plan-848-r3/{producer,consumer}`. Actual GPU-mutated
probe executables/PDBs are retained outside Git with hashes in
[native-fault-artifacts.json.txt](native-fault-artifacts.json.txt). Worktree,
consumer and caches remain for recovery. No deployment or batch-green claim.
