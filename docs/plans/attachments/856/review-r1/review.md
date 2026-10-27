# PLAN-856 independent final R1 — needs_fix

Reviewer: independent child context `/root/review_856`, did not participate in implementation. The inherited transcript was context only; verdict below comes from actual code, committed blobs, durable raw evidence and independent runtime counterexamples. Skill: `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md`.

Frozen tree: `D:/autostack/.wt/lang-856/auto-lang`, `plan-856-dev`, HEAD `3c39a96d21346e06b5c0ebf93b6e80ec4e5bb7a2`; base actual855 delivery `05fdc81c546aff735eefdb6c39fbc5fb35dc546a`. Status/HEAD/clean rechecked. SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`. No implementation, dependency or config edits. No whole-tier/workspace campaign, merge, canonical publication or cleanup.

## Findings

### F-856-R1-01 — P2: PixelDelta zoom uses physical pixels as logical pixels

Affected AC-02/05, V-01/V-04, T-03/04; T-05/06 closure evidence must be refreshed for this case. Original in-contract requirement: Plan5.2 says `pixel delta以40逻辑px换算1notch`; View2d/input must perform the physical→child conversion once.

Actual path: winit0.30.13 `MouseScrollDelta::PixelDelta(PhysicalPosition<f64>)` → bin `autographics-authoring.rs:204..209` copies raw x/y into iced Pixels → `Host::scroll`, viewport.rs:596 uses `y / 40.0` without physical-to-child conversion. Only pointer position is converted through CompositionInput. At physical window2400×1600 with fixed logical top1200×800, 80 physical vertical pixels are40 child logical pixels, so expected zoom1.1 from reset; actual1.21. DPI1/40physical control produces1.1. Document/history remain unchanged. Normal native DPI2 LineDelta check passes and does not exercise this branch.

Durable replay: [counterexamples.rs](counterexamples.rs), [replay.ps1](replay.ps1), [actual results](counterexamples.json.txt). Run the replay script from the authoritative tree after the named driver build; it links the frozen driver rlib and calls the same Host endpoint with the exact payload that bin forwards. This is not a claim that a Windows OS device delivered a PixelDelta during review; winit's actual physical payload type and the production forwarding branch are source-verified. The explicitly promised pixel-input branch is incorrect. Relevant winit source is in the dependency registry, event.rs:974; audit binds SDK/driver inputs.

Closing scope: convert pixel deltas into the same logical child units at exactly one boundary; retain Lines behavior, ±8logical-notch cap, finite/zero rejection and panel separation. Add permanent paired pixel controls at DPI1/1.5/2 plus Line controls, signed/large representative values and anchor preservation. Prove both the direct endpoint contract and actual bin handoff agree; do not merely divide the review fixture or weaken the Plan to physical units. Re-run affected native normal control after the repair; no unrelated suites are required.

### F-856-R1-02 — P2: navigation cancellation can fail to restore its starting view

Affected AC-05, V-04, T-03/04; T-05/06 closure evidence must cover the failure state. Original common contract: `Esc取消连续导航并恢复起始view，失焦/离窗/resize/DPI/关闭同样结束捕获并丢弃迟到事件`. No resource-failure exception is stated.

Setup: normal sample Host, initial frame, MMB down(300,190), move(340,210), frame. Current center(-40,20), starting center(0,0). Enable the existing `fail_resource` seam, cancel, clear fault, frame. Expected starting center/image; actual center.x=-40 and moved image remains. No-fault control restores center0 and exact original2D readback SHA. Blur and resize with the same fault reproduce the same failure; `fail_after_asset_upload` also reproduces it. All accepted document and history assertions remain intact.

Actual cause: navigation.rs:165..171 removes the navigation record and calls fallible `set_view2d(start)`. Its `prepare` encounters fail_resource (viewport.rs:17) or fail_after_asset_upload and rolls back to the last moved view; navigation has already been consumed. Unlike gesture restoration, navigation has no fault-independent restoration path. Thus capture ends but cancellation semantics are unmet. Raw original image SHA=`7ca9abd310619732c64bd073c0d068d30f673dd49e0e8ad4fd371eb1e6abec5f`; failed-cancel image=`c5d52db3d7b406af7cb7012cfa9b2bfd42ad36ae929d3b39767f25426cea28da`.

Durable same replay contains direct cancel, blur, resize and after-upload failure plus an accepted no-fault control. Esc/leave/close and failed-move recovery call this same cancellation function; source inspection establishes the shared path, without claiming every OS event was separately injected in the failure state.

Closing scope: make cancellation reliably restore the admitted starting view/frame despite the relevant preparation rejection, while ending capture and rejecting late input. Retain original faults/error provenance and resource/owner guarantees. Add permanent counterexample/control(s) covering both rejection seams and representative lifecycle callers, late move/up and actual restored content; ensure ordinary failed navigation updates still preserve their last complete frame. Replay original/fixed controls and current normal native Esc; no global test campaign is required.

## Acceptance map

| AC | R1 result | Independent evidence / boundary |
|---|---|---|
| AC-01 | pass | Exact branch/base/clean; current462 source hashes match committed Git blobs;681SDK source verifier and479package/17local/29edges closure passed. Locks/AGP/API/samples have no diff. Exported consumer identities and retained own-target native/selected tests bound in work raw;125 work evidence hashes independently rechecked. |
| AC-02 | fail | Default and unified CPU draw/pick/group/Gesture paths inspected; native20-stage normal passes. F-856-R1-01 violates logical pixel zoom. Focus/range/admission code and permanent assertions inspected. |
| AC-03 | pass within work scope | World-aligned1/2/5 spacing, bounded76lines/128capacity and2560CPUbytes inspected; scene/selection safe4096 bound; independent native grid readback passes. Raw20update owner/cost/default-comparison evidence hashes and assertions checked. No new view-owned VM/mesh/target owner. Full owner/content gates remain merge-owned. |
| AC-04 | pass within work scope | Render, exact reverse-draw ellipse/quad picking, group geometry/origin and real Auto Gesture consume the same view; final world snap and parent-local conversion, no-op/history and saved-content assertions inspected. Independent native Auto target/content/readback/Undo passes. No changed scene schema or Auto/API. |
| AC-05 | fail | Mutual exclusion/stamp/view checks and ordinary lifecycle/Play restoration inspected; independent OS F/Esc/Home/text separation passes. F-01 pixel boundary and F-02 failure-cancel restore remain open. |
| AC-06 | partial | New tool's normal/partial result and actual grid/Auto composite readback independently green; mutation reports are actual prior executables and nonzero named-stage results, hashes valid. Own-SDK/target consumer proofs and same-input default/performance claims checked with correct reuse boundaries. Oracle misses the two counterexamples; work cannot claim complete closure until permanent repair evidence. |
| AC-07 | fail | Final independent review returns needs_fix; fmt/diff clean; old SDK146 warnings separated from driver0 new. No unapproved production dependency/write scope or deferred feature workaround found. |
| AC-08/09 | pending merge | Full frozen producer+consumer and old/new native/owner/content, qualified AGP reuse, exact source landing onlyautographics-dev, canonical/ledger/archive/guard cleanup are not executed or authorized by this verdict. |

## Knowledge delta and routing

Proposed SD-01..04 targets and GOAL-008 are valid and scope-aligned; frozen proposed text is [spec-delta-frozen.md](spec-delta-frozen.md), hash in [audit](audit.json.txt). SD-01/03 currently state unconditional logical pixel/cancellation behavior which implementation does not satisfy. Repair implementation under the unchanged r1 contract; do not publish a weaker delta. SD-02 unified mapping and SD-04 source closure/input/oracle extension are otherwise grounded. No canonical Specs/ledger edits made.

T-01/02 remain complete. Reopen T-03/04 and their dependent T-05/06 closure, keep T-07 unpassed and T-08..10 pending. Plan becomes executing2/10. Next role: work in this exact tree, with the two finding rows and the replay/control inputs. Implementation writes remain paused until ownership is handed back. No final-review pass, landing, archive or delivery claim.

Raw review artifacts: `D:/autostack/plan-artifacts/plan-856/review-r1/`; independent normal+frame1partial, source check, counterexample executable/source/results and frozen bindings are indexed in audit.json.txt. The helper exits successfully after collecting cases; per-case `passed:false` rows are the expected product failures, not a green acceptance banner. No whole-tier cargo or full standalone suites were run.
