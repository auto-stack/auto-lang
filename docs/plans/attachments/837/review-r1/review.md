# PLAN-837 r1 independent final review — needs_fix

stage: review | review_scope: final | outcome: **needs_fix** | 2026-10-23 (Asia/Shanghai)

Reviewer: Codex in this review context, separate from the implementation handoff.
No extra agent/chat was created, and no model/session identity is asserted.
The executor's summaries were treated as claims and checked against committed
source, fresh tests, native execution and a reviewer-selected GPU counterexample.

Plan revision: 1. Reviewed commit: `abe682c5a1144f7e9b72f4ffeb95ece19dd02841`.
Base: `52ec4c7c13b337992278b343ef89d8aa17102ff6` (delivered PLAN-836).
Tree: `D:/autostack/.wt/lang-837/auto-lang`; branch: `plan-837-dev`.
Entry and exit checks confirmed the same HEAD and clean implementation tree.
All 30 changed files are under `experimental/autographics-auto-driver/`.
SDK: `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`; source/lock/Spec fingerprints
and review instruction sources are recorded in [baseline](baseline.json.txt).
Main has unrelated documentation WIP, which this review preserved.

## F-837-R1-01 / P2 — the native hierarchy gate accepts an invisible moved leaf

**Existing requirement:** AC-05/06/08, T-07 and dependent T-08, V-10 and §6 require
actual composed pixels consistent with world/picking, explicitly excluding
state/command-only closure. The readiness record additionally claims the expected
world endpoint matches the initial actual leaf color and the vacated location
matches background. This is a missing required oracle, not a new rendering feature.

**Cause established from code:** `src/bin/authoring_probe/hierarchy.rs:164–191`
checks accepted local/world/history, picking, and a matching GPU draw command,
then stores a capture hash. It never compares the moved endpoint pixel with
`self.color`, nor the vacated coordinate with `self.background`. The hidden-state
pixel check at line225 remains useful, but cannot establish visible-state content.
`tools/check_hierarchy.py:20–31` checks phase/exit and equality between saved and
reopened captures. The same wrong rendering in both processes passes equality.
`self.color` is collected and reported but has no post-move content assertion.

**Independent actual GPU counterexample:**

```powershell
python D:/autostack/auto-lang/docs/plans/attachments/837/review-r1/replay-pixel-oracle.py D:/autostack/.wt/lang-837/auto-lang/experimental/autographics-auto-driver
```

The replay compiles a review-only copy of the native entry against the frozen
driver/AGP libraries and original native probe modules. It changes no producer
source. After normal Host rendering, it makes the one moved circle command
transparent for a real GPU render, asserts that its actual composed center is
now background, and restores command inspection values. This models a downstream
render fault while retaining the real OS/Iced events, Auto output, local/world,
selection and history. Reopen receives the same faulty render. The production
check script is executed with only its executable path redirected to the review
binary; its assertions are unchanged. This is an injected validator counterexample,
not a claim that the unmodified product currently renders the circle incorrectly.

Expected: the composed-content oracle rejects the visible-state mismatch, native
positive exits nonzero / passed=false, and the aggregate gate fails.

Observed: the circle is absent in the [actual GPU capture](pixel-fault-native/positive.png),
yet native positive exits **0**, `passed=true`, phase16, 204frames. Reopen equality
and the complete positive/wrong-parent/drop-release/partial script all pass.
[Raw cases](pixel-fault-native/results.json.txt), [report](pixel-fault-native/positive.json.txt),
[replay log](pixel-fault-replay.log.txt), [harness provenance](pixel-fault-provenance.json.txt).
The initial standalone link attempt lacked a native Windows import-library search
path; its [raw setup error](pixel-fault-build-initial-error.log.txt) is retained.
Reading Cargo's native search paths corrected the harness; that setup error is
not classified as a product failure.

**Legitimate control:** the frozen unmodified binary completes phase16/204frames
and renders the circle correctly; wrong-parent fails phase5, missing release
fails phase9/timeout, frame1 fails incomplete. [Fresh native results](native/results.json.txt).
All38 targeted tests also pass. These results establish the coverage hole without
negating the behavior the current code does implement correctly.

**Closing oracle / scope:** add actual endpoint-color and vacated-location/background
assertions to the native composed-content gate, with hidden-state checks retained.
Have a real rendering fault such as transparent/misplaced descendant fail that same
gate despite valid state/commands, and propagate failure through report/exit/script.
Keep the unmodified positive, existing parent/release/partial negatives and saved
new-process reconstruction valid. Refresh the identity manifest, documentation,
handoff and proposed SD-05 if their assertions/inputs change. No expansion beyond
the existing finite2D/3D hierarchy profile is requested. This is blocking work;
recording it in KNOWN-DEBT is tracking, not permission to defer it.

## Acceptance audit

| AC | Result | Actual code/artifact and evidence |
|---|---|---|
| AC-01 | pass | G836 archive/merge receipt and actual base ancestry checked; SDK680/source245/479-package closure/17locals/29edges freshly verified in producer and independent consumer; no root/AGP/SDK/lock/game/asset change |
| AC-02 | pass | Document strict duplicate/unknown/missing-parent codec, v1 migration and finite forest validation inspected; fresh strict/deep128/shared-capacity/u64MAX/hidden-world tests pass, invalid graph rejected before Building/VM |
| AC-03 | pass | Local nodes feed real profile4 Session; complete count/ordinal/kind/dimension/parent/world/visibility/leaf fields independently admitted. Preview returned fields feed actual render/pick/outline. Wrong sum/visibility/parent/ordinal/profile controls and all-mesh3D matrix assertion freshly pass |
| AC-04 | pass | Actual Editor/Reparent/history and Host/group geometry/Gesture paths inspected. Keep-world precision rejection plus accepted control, subtree exact undo/redo/order/branch/high-water, busy/stale/last-endpoint,2D leaf/group and3D axes freshly pass |
| AC-05 | partial | Actual UI/pick/group gestures and normal native flow work; F-837-R1-01 prevents V-10 required composed-content gate closure |
| AC-06 | partial | Fresh v1 raw-byte protection, v2 nested new-process exact doc/world/unselected2D+3D captures/clean/empty history+selection and inherited save failure/epoch tests pass. Native paired captures can agree on faulty visible content (F-837-R1-01) |
| AC-07 | pass | Fresh20 hierarchy rounds and20 gesture/owner cycles check actual VM weak owners, mesh Arc/upload and target reuse plus last-owner external-pin controls. Work's actual retained-VM/fresh-mesh mutation evidence reused only for unchanged hashed sources; numeric/profile1/2 controls fresh2 pass |
| AC-08 | partial | Source/handoff/README and SD01..05 inspected. Finite scope, I2/I3 boundaries and exact commands are present; specific composed-pixel gate claims exceed the implemented oracle and must be reconciled after repair |
| AC-09 | fail | Final review produces one blocking required-oracle finding; cannot mark reviewed |
| AC-10 | pending | Merge-owned complete gates, landing, canonical/ledger deposit, archive, batch due check and guarded cleanup remain unexecuted |

Fresh targeted tests: **38 passed, 0 failed, 0 ignored**. Commands ran from the
authoritative driver directory with `RUST_MIN_STACK=16777216`:

```powershell
cargo test --locked --test authoring_hierarchy --test hierarchy_feasibility --test authoring_interaction --test authoring_probe_exit -- --test-threads=1
cargo test --locked --test authoring_document --test authoring_host --test authoring_session --test authoring_storage --bin autographics-authoring -- --test-threads=1
cargo test --locked --test numeric_probe --test squash_numeric_replay -- --test-threads=1
cargo check --locked --all-targets
cargo fmt -- --check
cargo rustc --locked --offline --bin autographics-authoring -- -C link-arg=/STACK:33554432
python tools/check_hierarchy.py D:/autostack/auto-lang/docs/plans/attachments/837/review-r1/native
python tools/verify_source.py
python tools/verify_plan837.py
```

[19 tests](scoped.log.txt), [17 supporting tests](scoped-support.log.txt),
[2 numeric tests](numeric.log.txt), [check](check.log.txt), [fmt](fmt.log.txt),
[source](source.log.txt), [external source](external-source.log.txt).
SDK146 inherited warnings remain; no driver warning was emitted. Formatting and
diff whitespace checks pass. New source was inspected for omitted paths,
unapproved deferrals/workarounds/debug output; no second blocking item identified.
Current unmodified native PNGs were visually inspected alongside the fault capture.

Work's source negative controls and actual resource-lifetime sensitivity receipts
are retained evidence, not counted as freshly rerun mutations: source fingerprints,
locks/dependencies, assertions/configuration and inputs are unchanged at this HEAD.
The same permanent positive lifetime tests were independently rerun here. Work's
old native property/836 compatibility receipts were inspected, while their full
refresh remains part of the merge matrix. No full driver/AGP/root test tier ran
during review. Full external consumer native/build gates likewise remain merge-owned.

## Spec delta and routing

[Frozen proposed delta](spec-delta-frozen.md) is byte-exact from the reviewed tree;
its hash and current canonical target hashes are in baseline.json.txt. SD01..04
match the inspected finite graph, storage, transaction and gesture semantics.
SD05's reproducible composed-pixel gate claim cannot be approved until
F-837-R1-01 closes. All seven superseded component paths are actual canonical
files; scene-hierarchy.md is the intended new component, and GOAL-007/008 are real
goals. I2 production AutoUI/RQHost, I3 Forge/distribution and graphics-only source
integration into autographics-dev remain accurately scoped. No canonical Spec
or live ledger was published by review.

Next: **auto-plan-work in the same tree**, repair F-837-R1-01, commit and refresh
handoff, then affected independent re-review. Plan returns to **executing6/10**:
T-07 reopened and dependent T-08 evidence reopened; T-01..06 remain completed.
T-09/10 are pending. The full frozen driver/AGP/16case/native/two-composite/source/
consumer merge matrix stays pending after a future final pass. No merge, archive,
branch/worktree removal or product-code repair was performed in this review.
