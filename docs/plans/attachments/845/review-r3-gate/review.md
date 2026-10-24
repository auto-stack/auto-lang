# PLAN-845 r2 — affected independent final review R3

**Verdict: pass**, bound to `a0e2a9115a0b48bab44e3acaef17485dca472420`. F-845-M1-01 is closed as a test-input visibility defect; the old gate is preserved. No product repair, changed contract, weakened assertion or new finding. Restore reviewed9/10 and T09 done; T10 and every complete merge gate remain pending.

stage: review | review_scope: final (affected repair) | plan_id: PLAN-845 | plan_revision: 2 | reviewer_identity: `/root/review_845_gate`, an authorized fresh context independent of implementation and the prior R2 reviewer; no further delegation or unexposed identity claimed.

Only existing implementation tree `D:/autostack/.wt/lang-845/auto-lang`, branch `plan-845-dev`, was inspected/tested. Entry and final checks find clean HEAD above, parent `e518f5fc778cfa6827f7de9eb99b4c0910ef294d`; original base remains `8b62db5cba54b9bfe7f759f4920b22ccb9b75e88`. Read the specified auto-musk review skill and verification-contract reference, repository AGENTS.md, current shared Plan and R2 receipt. This review writes only main Plan bookkeeping and this evidence directory. No implementation writes, merge, canonical publication, archive or cleanup occurred.

## F-845-M1-01 closure

Existing invariant: preserve original real Host/Iced create/select/delete/bad-Open gate and its assertions, AC07/08 and T08, without replacing actual UI input by direct dispatch. The failed old complete driver invocation is retained in [driver-test.txt](../merge-r1/driver-test.txt): actual_create_delete_widgets_and_bad_open expected5, observed4 after the create click. The other six authoring_host tests passed. This is retained failure evidence, not a successful complete gate.

I read the complete two-file [repair diff](repair.diff.txt), original and current test, actual Host widget_bounds, scroll, click/frame paths and fixed viewport. Host uses1200×800 logical coordinates and panel starts at x810; widget_bounds subtracts the actual observed scroll offset. Scroll queues actual Iced wheel input at the panel cursor; frame processes it. The helper only observes current widget geometry, sends bounded wheel input and then the same real click. It retries at most30 times, panics on unreachable geometry, and unwraps missing layout. No dispatch, editor mutation, assertion suppression, ignored test, changed expected count or pixel tolerance was introduced.

All original assertions remain: create publishes through Auto to5 objects, actual node5 click selects5, delete publishes through Auto to4 and clears selection, failed Open preserves exact Document and history, and actual captured pixels are equal before/after bad Open. Only the three original click-location blocks now use visible geometry. The helper also handles reverse scrolling after create/selection changes the layout; this is exercised by the actual sequence, rather than assuming the first scroll offset remains valid.

| Check → finding/invariant | Independently observed result |
|---|---|
| Unchanged durable real Host probe → input reaches claimed UI state | Recompiled [prior probe](../merge-r1/create-visible.rs) against the frozen library. Unscrolled action-create y889.5998 lies beyond800; actual click leaves4 objects. Scroll/frame places it at y709.5998; actual click creates5. Actual node5 selection and delete return4 with no selection. [Replay](replay-visible.ps1), [raw](visible.log.txt), [exit0](visible-exit.txt). |
| Original permanent entry → old oracle retained | `RUST_MIN_STACK=16777216 RUSTC_WRAPPER='' cargo test --locked --manifest-path experimental/autographics-auto-driver/Cargo.toml --test authoring_host actual_create_delete_widgets_and_bad_open -- --exact --nocapture`: 1 selected/1 passed/0 ignored,6 filtered; [raw](permanent.log.txt), [exit0](permanent-exit.txt). |
| Exact source/Spec/dependency baseline → retained acceptance validity | [Audit](audit.json.txt), [replay](audit.py): all320 committed source objects match current manifest; parent-to-HEAD changes exactly authoring_host.rs and its single source-manifest hash. Three locks, configuration, skill inputs and all16 R2 canonical Spec inputs match R2. Spec proposal unchanged. Actual SDK verify-only passes681 files/17 local packages. |

Independent probe output reproduces both ineffective old offscreen input and successful visible input on the same frozen production code. The permanent real test closes the original oracle end to end. These observations establish test-input positioning as the cause; they do not infer create success from counts in a mock. Probe source identity is included in audit. No mutation of committed implementation was needed or performed.

## Reuse and acceptance boundary

R2's [full independent receipt](../review-r2/review.md) remains the evidence map for AC01..08 and SD01..04. This parent-to-HEAD repair changes no production Rust, SDK, AGP, lock, fixture, sample, tool, native input, ownership behavior or Spec proposal. Current source identity differs only for this permanent test; dependency/configuration/Spec identity is independently checked above. Therefore AC01 source baseline is refreshed; AC07/08 original-gate preservation is freshly verified; AC02..06 and unrelated AC07/08 evidence remain pass by explicit identity reuse. AC09 passes here; AC10 remains pending. F-845-R1-01 stays independently closed by R2, with its same production/source-state tests unchanged; no replay of that closed finding or unrelated GC/native/consumer families was warranted.

The frozen [Spec proposal](spec-delta-frozen.md) is exact committed bytes, SHA256 `01eaa3e632a062c50bbcd5860df79d5a28e4d56390c881711754fb6416a351c5`, equal to R2. Its existing source-origin rule, single-blob contract, limits and production boundary remain valid. Exact target paths and GOAL-007/008 metadata are unchanged. No canonical Spec or ledger was edited.

Health: package fmt and `git diff HEAD^ HEAD --check` pass. Targeted test builds the changed test and produces no new driver warning; immutable SDK's146 previously disclosed warnings remain. Source320 filesystem verifier and pinned SDK681 verifier pass. No debug prints or dropped assertions were added to the test; no contract item was deferred as debt. The existing closeout finding is closed here, with no new debt candidate.

## Routing

All complete merge gates remain pending on this newly reviewed freeze: original63 invocations; full locked driver/AGP test/build,16 AGP/dual games, original authoring/hierarchy/assets/Play/source/owner/reopen/counter AND content controls; producer/fresh own-SDK/own-target consumer;15 project-native cases and actual UI-saved project relocation. Earlier aborted complete driver run does not fulfill them. Merge owner reruns invalidated complete gates on this reviewed commit, then lands only autographics-dev and performs knowledge/ledger/index/archive/guard cleanup and845 batch-due handling. Root cargo tiers/docs_gen remain untriggered by this driver-only repair; tf stays main single-instance batch-only.

No full driver/workspace/root/native/GC campaign was run in this review. This is affected final acceptance pass, not final delivery. Final source-tree check remains clean at the reviewed HEAD.
