# PLAN-829 — independent final review R1

Date: 2026-10-21 (Asia/Shanghai). **Outcome: needs_fix**. Scope: final.
Reviewer: Codex in this review chat; independent of the implementation chat.
No second agent/model/session was invented or created. Source writes were paused;
the committed implementation was inspected and tested in its existing tree.
Review-only injection copies live in this attachment, not in implementation files.

## Frozen baseline

- Plan revision: 1. Entry status execution_done, 9/11; input Plan SHA256
  `16aebb672eff11b7364c444693d11f2a3604bad68af8bb30a2faa3bdcccce01d`.
- Worktree: `D:/autostack/.wt/lang-829/auto-lang`; branch `plan-829-dev`.
- Reviewed commit: `f7ea95f3b71459914cc081d70dd62123c1303a6d`.
- Base: `fdb1fad2ce7afa124adfa3080323496df384e670` (verified merge base).
  Current main at entry: `9b65383e0`; main has unrelated document WIP,
  preserved. Implementation worktree clean at entry and final baseline check.
- Diff: 174 added files, only `experimental/autographics-prototype/` (AGP).
  Root workspace/lock and production crates unchanged; reparse-point scan empty.
- Dependencies: AGP own lock; wgpu 27.0.1, winit 0.30.13, iced/iced_wgpu 0.14.0,
  iced_widget 0.14.2 path patch. Cargo.lock SHA256
  `d9e1fdae2c2d6d299983796f766442599dac9482dd79575942f2db85c3bee235`.
- Relevant Spec inputs: main `docs/specs/overview.md`,
  `docs/specs/auto-lang/ui/overview.md`,
  `docs/specs/repository-boundaries/project.md`; Design45 and current Plan.
  Exact hashes, including the pre-review Plan: [input-sha256.txt](input-sha256.txt).
- SD-01/02/03 exact proposals frozen beside this report as `spec-delta-*.md`.
  Their SHA256s respectively:
  `82b2a82ca5c43b7694508edd87492d53b594120bd871689d1136572b101804e7`,
  `78f139ecd662732557afec8a36078c89792f19f875ab9d0ca58c399787960804`,
  `1010f8caee762198c62ab11cf1c21333df62a04a41e87ed482ff6070bd415738`.

## Reproduced evidence

All commands below ran from the implementation tree; `M` means
`experimental/autographics-prototype/Cargo.toml`. No root t/tv/tt/tb/tu/taa/tf
or docs_gen was run. The independent workspace's acceptance tests are required
targeted review checks, not the root regression tiers.

| Check | Result / coverage |
|---|---|
| `cargo test --manifest-path M --locked` | exit 0, **74** passed: lib17 + breakout11 + prototype16 + squash8 + threed10 + verifier7 + iced4; no ignores; four compiler warnings below. Executor's 73 count is stale. |
| `cargo build --manifest-path M --locked --lib --bin graphics-probe` | exit 0, frozen library and debug runner produced for independent probes |
| `cargo metadata --manifest-path M --locked --offline --format-version 1` | exit 0; independent dependency closure inspected; patch remains inside AGP |
| asset/source/font SHA256 verification | 6 assets + 22 frozen source texts + 2 font files = 30/30 match |
| `cargo fmt --manifest-path M -- --check` | exit 1, owned source/test formatting differences; [raw](fmt-output.txt) |
| `git diff --check BASE...HEAD` | exit 2, frozen font-license and copied widget trailing whitespace; [raw](diff-check-output.txt) |
| real runner `ag2d-breakout`, canonical, headless | exit 0; [report/keyframes](breakout/report.json.txt), 10800-tick reference, scoring/audio chain. First-brick PNG visually inspected. This is not proof of the LIVE loop or source-image comparison. |
| real runner `ag3d-squash`, canonical, headless | exit 0; [report/keyframes](squash/report.json.txt), 3600 ticks, 47 events, 3 squashes, music Playing. Squash PNG visually inspected. Source fidelity gaps below remain. |
| real runner `lifecycle`, canonical, headless | exit 0; [report](lifecycle/report.json.txt), misleading release fields independently challenged below |
| real runner `metrics`, canonical, headless | exit 0; [report](metrics/report.json.txt); CPU timing is actual wall clock, GPU timing honestly unavailable; accounting gaps below |
| real runner `verifier-negative-controls`, canonical, headless | exit 0; [report](negative/report.json.txt), five existing controls passed; these do not cover final composition or actual cleanup |
| real runner `iced-shared-device`, canonical, window | exit 0; [report/keyframes](iced-positive/report.json.txt), real event-path click, DPI, resize. Actual final composition and shutdown not established by that verdict. |
| independently linked behavior probe | exit 0 means **defects reproduced**, not accepted behavior; [source](behavior-probe.rs), [stdout](behavior-output.txt) |
| lifecycle copy retaining every temporary target | exit 0 / PASS despite 20 unreleased targets; [mutated source](lifecycle-leak-injection.rs), [report](lifecycle-leak/report.json.txt) |
| Iced copy skipping final composite draw | exit 0 / PASS despite neither graphics view being drawn into the window; [mutated source](iced-no-blit-injection.rs), [report](iced-no-blit/report.json.txt) |

Replay invocation for ordinary cases:
`./experimental/autographics-prototype/target/debug/graphics-probe.exe --case CASE --replay canonical --report D:/autostack/auto-lang/docs/plans/attachments/829/review-r1/CASE --mode headless`
(Iced uses `--mode window`). [replay.ps1](replay.ps1) preserves the exact R1
library-link/injection setup. Re-run its reproduction assertions against R1;
after repair, regenerate injections against the new source and replace defect
assertions with contract assertions. Do not transplant the old copied runner
and claim it tested newly repaired code.

The four other runner cases (gpu-foundation, shared-runtime, ag3d-risk-slice,
independent-consumer) were source-audited and their existing research/receipts
read, not freshly rerun. Source/model/consumer checks above cover part of them;
full runtime acceptance remains pending on the repaired baseline. In particular
the expensive relocation rebuild was not repeated to reconfirm an already
known non-pass. No V-01 resize/minimize or A→B runtime pass is granted this round.

报告快照以 `report.json.txt` 保存，避免仓库对 `report.json` 的忽略规则丢失证据；运行时原 JSON 仍保留。格式日志仅保留截断诊断，不将全量运行日志入库。

## Findings (all open)

### F829-R1-01 — P1: LIVE Squash cannot accept the promised jump input

`AGP/src/host/cases/squash_game.rs:1089,1123`: Space and Enter are handled only
as retry when dead. During every living redraw `action.jump = true` is
unconditional. No jump-key state or edge is captured. A player automatically
jumps whenever grounded without pressing Space; pressing Space while living
does not control the jump. The frozen `Player.gd:39` explicitly uses
`is_action_just_pressed("jump")`. Canonical JSON actions bypass this code,
so the green trace cannot prove the interactive requirement.

AC-04 / T-06. Repair the real keyboard path (press edge, release, grounded/
airborne/dead/retry states); cover no-input idle and an actual single Space
press, including held-key behavior. Do not replace the source oracle with
automatic jumping.

### F829-R1-02 — P2: both LIVE games advance simulation by redraw count

`AGP/src/host/cases/breakout.rs:1063..1096` advances one 1/60 tick per redraw;
`squash_game.rs:1110..1161` advances two 1/120 ticks per redraw. Neither uses
elapsed time or an accumulator. Sleeping 8ms is not a 60Hz clock and cannot
decouple physics from display rate/load. At 30 redraws/s these games simulate
only 0.5 seconds per wall second; at 120 they simulate 2 seconds. This violates
the frozen 60/120Hz profile and manifest's rendering-decoupled contract even
though every individual tick is deterministic. This arithmetic is source
evidence, not a claimed measured desktop refresh rate.

AC-03/04 / T-04/06. Drive fixed ticks from elapsed time with a documented
catch-up policy; verify representative redraw cadences produce the same
physics state after the same elapsed interval. Recheck resize/zero-size
handling in these LIVE hosts while exercising that actual path.

### F829-R1-03 — P2: required Squash source-state paths are missing

- **Spawn target** (`games/squash/sim.rs:197`): dx/dz target literal zero.
  Frozen `Main.gd:36` passes the player's current position and `Mob.gd:18`
  aims at it. Same-seed independent runs with stationary vs right-moving
  players yielded x=0 vs **7.116672**, but identical first yaw **-0.6665616**.
  This is a semantic mismatch, not a numerical integrator tolerance.
- **Death detector** (`sim.rs:288`): all contacts use the body sphere/OBB;
  the five separate MobDetector cylinders from `Player.tscn:63..83` are
  absent. The body-contact classifier is not equivalent to the Area3D
  signal path documented in evidence-notes E3/E4. In the preserved probe,
  mob at (0.716365,1,2.1) intersects the frozen forward detector cylinder
  (center 0.716365,0.95,1.71889; radius .2; height .144), but the player
  sphere does not touch the mob box; actual `dead=false, events=[]`.
  The expected detector overlap follows from the frozen geometry; this
  probe did not execute Jolt and does not claim full Jolt equivalence.
- **Facing/animation** (`host/cases/squash_game.rs:194..198`): player model
  has no movement-facing yaw, bob is always global time ×4, and the parent
  jump-arc rotation from `Player.gd:66` is missing. Source sets facing on
  movement, animation speed 4 when moving / 1 when idle, and pitch from vy.
  Mobs similarly use global elapsed time instead of age since spawning.

AC-04 / T-05/06. Implement the separate source-owned states and channels,
then verify moving-target spawns, detector-only side contacts, top landing
controls, idle/moving facing and animation, spawn-local animation and jump
pitch. A documented single-pass physics approximation does not authorize
deleting these non-equivalent state/input paths.

### F829-R1-04 — P2: cache accepts stale ownership and leaks resource identities

`AGP/src/renderer/resources.rs:98`: only the supplied owner is validated;
CacheKey's owner string and the entry's owner are never matched. Allocate A,
cache A's 16×16 texture, free/reuse A as B, then request A's key with valid B
and size32×32: **accepted, created stays1**, returning the old texture. Fresh
B key succeeds and actual stale A identity is rejected (positive controls).

Separately, every new entry allocates an ID at line107, but eviction/release
at lines154/163 never frees it. Capacity2 with 64 create/release cycles grew
allocator live count **3→67** while live textures ended at1.

AC-02/05 / T-02/07. Bind keys/entries to the validated owner/generation;
define same-key size changes; free entry IDs on both eviction and explicit
release. Check sibling failure/resize/reset paths and bounded warm baselines.

### F829-R1-05 — P1: Iced gate accepts missing final composition

Official `iced_shared_device.rs:59..131` checks offscreen texture captures,
flags set unconditionally in `prepare`, and manually assembled token
strings (`:279`). It never verifies the window composite, clipping, alpha,
or the identity of the target actually drawn into that composite.

**Executed mutation:** copy iced_spike.rs, replace only
`pipeline.blit(self.kind, render_pass)` by `let _=(pipeline,render_pass); true`.
Unmodified official case logic still produced **exit0, exit=pass,
all_gates_behaved=true, real click=true, views_on_iced_device=true**.
The offscreen views continue updating while both final draw calls are gone.
The injected runner is isolated to review copies; original source stays clean.

AC-07 plus AC-05/06 / T-08/07. Add assertions against the final composed
window/render target (controls, both views, clip boundary, alpha, DPI and
resize), with actual blank-final-draw and foreign-target/device mutations.
Derive device evidence from resources used on the path, not synthesized
matching strings. A valid unchanged real-window positive must still pass.

### F829-R1-06 — P2: lifecycle release/close oracle is not observing teardown

`AGP/src/host/cases/lifecycle.rs:64..77` collects both the before and purported
after RSS/thread samples **before the workload**; later code reuses the
early third sample as "after". Thus the return-to-baseline narrative is not
an after-teardown measurement. View dropped count is incremented by2 and
`live_targets_after=1` is literal, rather than derived from owned lifetimes.

**Executed mutation:** add `std::mem::forget(t1); std::mem::forget(t2)` just
before `dropped += 2`. All 20 temporary targets remain owned/leaked, but the
official gate still exits0 with `baseline_restored=true, targets_dropped=20,
live_targets_after=1`.

The 20 game rounds use pure drivers and a separate synthetic cache; Squash
counts an always-empty Runtime scene, not actual driver mobs/meshes/audio.
Dead mobs are retained in its Vec. The map_async callback ignores outcome;
`clean_exit=true` is filled before device/window destruction and is absent
from `all_ok`. The Iced canonical path calls `std::process::exit` before
ordinary window/GPU/static-store destruction (`iced_shared_device.rs:41`).

AC-05/07 / T-07/08. Observe real owned resources/tasks before/after GPU
completion and teardown, integrate the actual game's resources in restart
cycles, await cancellation acknowledgment, and let the host close normally.
Use an external supervisor for exit/worker evidence. Intentional retained
targets, failed cancellation and blocked shutdown must make the gate fail.

### F829-R1-07 — P2: required source oracle / fidelity validation is incomplete

T-01 promises original-sample run/inventory and frozen source-based
oracles; the delivered evidence is source parsing plus new Python reference
implementations, not original-engine baseline runs. The latter are useful
for determinism, but cannot establish that the approximations match Godot.
The stationary reference also repeats the origin-target assumption missed
by F03. Pixel tolerances in sample-manifest §1.5/§2.6 are never exercised:
game `build_report` accepts the right number of merely **distinct hashes**;
no source golden image or declared content oracle is compared.

The official Squash runner calls an event sequence "exact" after comparing
only tick+kind (`squash_game.rs:336`); `spawn_stream_matches_golden` compares
only spawn **counts** (`:409`). Unit tests cover more fields but omit vy and
contain an unreachable per-event score assertion. The binding manifest
still says 30 physics ticks=0.5s at120Hz and seed0x8293D01, while code/trace
use60 and seed0x8, without a reconciled current oracle contract.

AC-01/03/04/06 / T-01/04/05/06/07/09. Reconcile the oracle with immutable
sources and preserve actual source-run/reference evidence or an explicitly
approved alternative. Compare required gameplay/event payloads and visual
content against that independently established oracle; freeze/review any
semantic correction. Do not make the implementation its own definition of
fidelity or silently lower image assertions to uniqueness.

### F829-R1-08 — P2: measurement accounting and evidence identity are incomplete

`AGP/src/host/cases/metrics.rs:111,130,193,212`: 2D uploads are literal1,
3D only measures MeshStore entry-count change, and rebuilds are literal0.
The definition alternates between resource creation and bytes rewritten:
QuadPipeline rewrites an existing buffer, whereas MeshPipeline rewrites
camera/light/instance buffers every frame (`render3d/pipeline.rs:568,582,601`)
yet is reported upload-free. Warm loops recreate pipelines/targets each pass
despite methodology claiming already-resident pipelines. Frame samples are
discarded after summarizing; no raw series to recompute quantiles survives.

`host/report.rs:87..140` lacks commit/lock/asset-set fingerprints; metrics
itself contains none. Reports always say pass after successful JSON writing,
including missing RSS (`-1`) or accounting assumptions; existing negative
controls do not test metric-measurement failure. CPU wall timing is real and
GPU timing unavailable is correctly disclosed; those parts remain useful.

AC-06 / T-07/09. Define separate measured creations/rebuilds/uploads (and
bytes where needed), instrument actual paths, keep raw measurements and
fingerprints, and ensure missing required observations/failed accounting
cannot produce PASS. Correct the cold/warm description to match execution.

### F829-R1-09 — P2: vendoring contradicts the approved patch-source boundary

Plan §3 requires public Iced entry/necessary existing patch **source**, and
explicitly says "不复制 patch 源或 UI 代码"; V-09 says patch resolves through
recorded source/external location. The implementation adds the entire
`AGP/patches/iced_widget/` source tree and Cargo path patch. PROVENANCE.md
claims T-08 sanctions copying, but the shared r1 Plan contains no such
approval. No-link and no-private-runtime-dependency checks passing do not
authorize this different delivery model.

AC-01/08 / T-08/09. Restore the agreed source/provenance dependency model;
if vendoring is necessary for relocation, return to new for explicit contract
resolution rather than asserting that approval already exists. This review
does not silently approve or prescribe an untested alternative source.

### F829-R1-10 — P3: health gate and result/spec statements need correction

Test compile warnings: verifier_contract.rs:67 unused driver_w;
squash_contract.rs:120 unreachable score match, plus unused vy and spawn
index/tick/vel fields. The unreachable branch is missing verification, not
just style; restore its intended assertion instead of suppressing it.
Owned Rust source/test fmt check fails (truncated diagnostic log retained). Report says
73 tests/zero warnings but actual tests are74/four warnings. Preserved
upstream/license bytes also have diff-check whitespace; handle their
provenance explicitly rather than rewriting source/license bytes casually.

SD-02 asserts verified cleanup/no owned workers though iced_spike creates
a thread at line822; SD-03 asserts verified composite/exit based on the
false-positive gate. integration-handoff.md:23 still claims foreign binds
were rejected, while the actual implementation/report say they were not.
These are contradictory current facts, not safe future Spec text.

AC-08/09 / T-07/08/09. Clean owned formatting/warnings and the missing test
assertions; reconcile result/measurement/handoff/SD statements with actual
observations after repair. Do not publish these proposed facts as current.

### F829-R1-11 — P2: 3D bypasses the agreed common scene/identity/change contract

`AGP/src/games/squash/sim.rs` updates only SquashDriver fields/mobs Vec and
emits events; it never submits shared scene changes or assigns shared object/
resource IDs to player/mobs. `frame_instances` reads those private fields
directly. Runtime scene and ChangeLog remain empty while the rendered 3D
world changes (lifecycle baseline0 is explicitly documented). A generic
consumer replaying the shared model/log cannot reconstruct or update this
game, and invalid-ID/local-edit controls do not reach its objects.

Plan §2/§5.2 requires shared model/object/resource identity, scene changes
and time/events, with game logic submitting changes to the renderer; 2D and
3D are not authorized to use separate object/state contracts. Correctly
tested generic Runtime primitives alone do not prove the Squash integration.

AC-02/05/08 / T-06/09. Connect the 3D path to the agreed shared identity/
delta contract (dimension-specific algorithms can remain separate), and
verify an independent consumer reconstructs/edits the actual 3D scene,
including stale updates across restart. The current T-03 primitives/tests
remain accepted as partial reusable foundation; this is missing end-to-end
integration, not a request for a new generic ECS.

## AC → task → implementation → evidence disposition

| AC | Result | Current evidence and remaining gap |
|---|---|---|
| AC-01 | partial | own workspace/lock and 30 hashes pass; source oracle and patch authorization F07/F09 open (T01/02/09) |
| AC-02 | partial | generic Runtime tests and real GPU rendering exist; cache ownership F04 and missing 3D model connection F11 prevent complete G-base acceptance (T02/06) |
| AC-03 | partial | canonical 2D playback/HUD/audio path reproduced; LIVE timing F02 and source image oracle F07 open (T04) |
| AC-04 | fail | canonical3D renders/audio/replay pass against local approximation; real input F01, timing F02, source-state F03 and oracle F07 fail (T05/06) |
| AC-05 | fail | existing stale-ID controls pass, but F04 cache lifecycle and deliberately leaked GPU targets F06, blank-composite false-positive F05, shared3D F11 remain (T02/06/07/08) |
| AC-06 | fail | CPU wall values real/GPU unavailable honest; non-discriminating acceptance F05/F06, incomplete metrics/provenance F08 and oracle F07 (T07/09) |
| AC-07 | fail | real Iced positive reproduces, but no-final-draw mutation also passes; teardown unproven F05/F06 (T08) |
| AC-08 | partial | IH01..05 owners/gates and I1..I3 pending appropriately retained; unauthorized patch F09, false current facts F10, missing shared3D consumer F11 prevent pass (T08/09) |
| AC-09 | fail | this independent review executed; found in-contract open findings; T10 remains unchecked |
| AC-10 | pending merge-owned | not attempted; requires repaired final acceptance and migration handoff authorization first |

## Spec delta review

Targets SD-01 project.md, SD-02 prototype-profile.md, SD-03 integration-contract.md
are valid future `docs/specs/auto-graphics/` paths, currently absent. Proposed
new-module impact and GOAL-008 are retained; no superseded components. Existing
UI/repository-boundary Specs remain authoritative and unchanged.

- SD-01 partial: scope/entries and I1–I3 pending are appropriate, but patch
  delivery and claimed source-fidelity evidence need F07/F09/F11 resolution.
- SD-02 fail: measured lifecycle/cache/model/metric claims affected by
  F04/F06/F08/F10/F11, source-based controls incomplete.
- SD-03 fail: future integration boundaries sensible; "verified composite/
  resize/DPI/exit/second-device" exceeds actual discriminating evidence F05/F06.

Frozen proposals above are **rejected current-fact proposals**, not approved
text for deposition. No canonical Spec or ledger was published in review.

## Readiness / routing / pending closeout

Original requirement gaps F01..09/F11 and health/closeout defect F10 are
within r1; recording debt does not authorize deferral. No silent workaround
or alternate profile accepted. No semantic Plan revision was made in review.

Reopen T01,T02,T04,T05,T06,T07,T08,T09 (8 tasks). Preserve T03's independently
verified generic core primitive progress and its historical evidence; the
3D connection remains T06 work. T10/T11 unchecked. Status executing,
**current_step1 / total_steps11**. Return to work in the **same** tree; repair
all findings as one coherent batch, run targeted counterexamples/acceptance
checks, commit, reconcile the closure matrix and submit a new frozen SHA.

Merge-owned gates remain **pending**: final pass on repaired code; frozen
independent-workspace applicable tests/probes and real consumer relocation;
migration-completion/write-surface handoff explicitly permitting landing;
canonical Spec deposition + derived ledger reconciliation; archive and
wt-guard-clean cleanup. No root tiers apply while only AGP is touched.
No merge, archive, branch removal, implementation fix, or external publication
was performed by this review.
