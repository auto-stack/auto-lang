# PLAN-829 independent final re-review R4

2026-10-21 (Asia/Shanghai). **Outcome: needs_fix.** Plan revision 1;
review_scope final. This is the original R1 review chat, independent of the
implementation chat. No new agent/chat was created. R2/R3 receipts were read
as prior evidence; their PASS/closure labels were checked against actual code.
The explicit user request authorizes this re-review of the active unlanded Plan.

## Baseline and preserved evidence

- Existing tree: `D:/autostack/.wt/lang-829/auto-lang`, branch `plan-829-dev`.
- Reviewed HEAD **077de3b6697c69053c24f4067bf73188ca90ab62**; base
  **fdb1fad2ce7afa124adfa3080323496df384e670** (merge base verified).
- Entry status reviewed, 10/11. Input Plan SHA256
  `0f781a452910a854d05d4b28c9dc03a400c4419fb62e4f76227f47b8dd6e6fb6`.
- Worktree clean at entry and final source check. 123 changed files, all
  AGP (`experimental/autographics-prototype/`); no root workspace/lock or
  production-code changes. Reparse-point scan empty.
- Dependencies: wgpu27.0.1, winit0.30.13, iced/iced_wgpu0.14.0,
  **stock** registry iced_widget0.14.2. No copied patch tree remains.
  Cargo.lock SHA256 `6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`.
- Relevant canonical inputs: specs overview, goals (GOAL-008 verified), UI
  overview, repository-boundaries/project. Auto-graphics Specs do not yet
  exist. Their sources, Plan, lock, affected code and SD proposals are hashed
  in [input-sha256.txt](input-sha256.txt). SD-01..03 exact proposals are frozen
  as `spec-delta-*.md` beside this report. No canonical Spec/ledger published.

`report.json.txt` files are byte copies of actual JSON outputs, retained under
trackable names because repository rules ignore `report.json`. PNGs are actual
captures. Full repetitive build logs were not copied into the repository.

## What was reproduced on this commit

All commands ran in the existing implementation tree. `M` denotes
`experimental/autographics-prototype/Cargo.toml`.

| Check | Actual result |
|---|---|
| `cargo test --manifest-path M --locked` | **90/90**, exit0, no compiler warnings: lib24, breakout11, cache3, iced5, prototype16, squash14, threed10, verifier7 |
| `cargo build --manifest-path M --locked --lib --bin graphics-probe` | exit0; current library/runner used by probes |
| `cargo fmt --manifest-path M -- --check` | exit0 |
| `git diff --check BASE...HEAD` | preserved upstream FiraMono license trailing-space exception only ([excerpt](diff-check.txt)); no new owned-source formatting issue |
| Asset/source/font/golden hashes | **36/36** match ([results](asset-check.txt)); integrity alone does not prove source fidelity |
| `ag2d-breakout --mode headless` | exit0; [report](breakout/report.json.txt); R2 offscreen-oracle wiring defect no longer reproduces |
| `ag3d-squash --mode headless` | exit0, position diff0, full event/spawn checks green, content oracle green; [report](squash/report.json.txt). **Nevertheless renders a player after death**; [actual retry frame](squash/retry_overlay.png) |
| `lifecycle --mode headless` | exit0; [report](lifecycle/report.json.txt); real-resource/witness/callback/sampling repairs are present |
| `metrics --mode headless` | exit0; [report](metrics/report.json.txt); current commit fingerprint, raw-series/accounting fields and CPU measurements present; GPU timing explicitly unavailable |
| `verifier-negative-controls --mode headless` | exit0; [report](negative/report.json.txt); five existing controls green, coverage does not include the new counterexamples |
| `iced-shared-device --mode window` original executable | exit0, real click and resize; [report](iced-positive/report.json.txt). Unlike R3, this round's original windowed positive actually ran successfully |
| Independently linked public-API behavior probe | exit0 means **four defects reproduced**, not contract acceptance; [source](behavior-probe.rs), [stdout](behavior-output.txt) |
| Isolated-title Iced control, original shader | exit0/PASS; [report](iced-unique-positive/report.json.txt) |
| Same Iced control, final fragment forced transparent | **exit0/PASS**, all_gates_behaved/composite.ok true; [report](iced-unique-transparent/report.json.txt) |

Ordinary runner invocation:
`./experimental/autographics-prototype/target/debug/graphics-probe.exe --case CASE --replay canonical --report <this attachment>/CASE --mode headless`
(Iced uses window). This review did not run root t/tv/tt/tb/tu/taa/tf/docs_gen,
merge gates, or a fresh relocation rebuild. R3's two consumer replays are
retained only as historical same-SHA evidence, not relabeled as new executions.
GPU-foundation/shared-runtime/risk-slice have code/test/prior-receipt inspection
this round; their complete runtime matrix must be checked on repaired code.

### Injection validity and environment boundary

Review code was compiled against the current library, with dependency paths
resolved from current Cargo JSON artifacts ([identities](link-artifacts.json.txt)).
The first Iced injection accidentally selected stale cached iced/image rlibs
and timed out; it is **invalid baseline evidence**. A corrected same-title
attempt also timed out waiting for input. Both records are retained and neither
is treated as a product rendering failure; only their owned process trees were
terminated. See [attempt note](injection-attempt-note.txt) and status files.

To isolate the same-title click routing, both conclusive controls use unique
window titles consistently in the app and its own clickers. The unchanged
case/verdict logic and all dependencies are identical. Both controls receive a
real winit input event, perform resize and return normally within the external
25s deadline. The sole rendering change in the negative control is:

```wgsl
// positive
return vec4<f32>(c.rgb, 1.0);
// negative: final UI composite only; offscreen scene textures remain intact
return vec4<f32>(0.0, 0.0, 0.0, 0.0);
```

Sources: [positive](iced-unique-positive.rs),
[negative](iced-unique-transparent.rs),
[unchanged case](iced-case-original.rs); each has a corresponding `*-main.rs`.
The renderer has ALPHA_BLENDING, so the negative fragment contributes no visible
graphics to the final window while real draw calls/counters keep advancing.
No implementation source file was mutated. [replay.ps1](replay.ps1) reconstructs
the current-dependency runs; frozen source copies must be regenerated against
future repaired code rather than reused to assert closure.

## Five concrete findings

### F829-R4-01 — P2: deleting the player changes a mob's entity type

Location: `AGP/src/games/squash/sim.rs:310..322`, consumed by
`host/cases/squash_game.rs:189..202` in both windowed/offscreen paths.
`project_instances` chooses the **first currently alive scene object** as
Player, although its comment promises a stable first-spawned player identity.
After player destruction, the first surviving mob is therefore rendered using
player meshes. Classification changes again as the leading mob disappears.

**Bound input:** current canonical `golden_squash_trace.json`, seed0x1B,
unchanged public Runtime/SquashDriver, step through tick1372. **Expected:** dead
player ID destroyed, all remaining projected entities Mob. **Observed:**
`dead=true`, actual player ID `(index0,generation0)` invalid, four live mobs,
but `(index6,generation4)` classified Player. The actual official game runner
also exits0; the retry-overlay PNG visibly contains the orange player model.

Cause established: deriving semantic type from the mutable vector head.
This is a regression introduced by the F11 shared-scene repair, not the old
private-driver representation. The existing F11 test stops at600 while the
player is alive and misses the deletion state.

AC-02/04/05/08; T-06/07/09. Preserve entity type/identity through the shared
model/log and deletion/reuse. Close with before-death, after-death-with-mobs,
leading-mob deletion, empty/restart and independent-log-replay controls.
Repair the source-derived content expectation before freezing new frames.

### F829-R4-02 — P2: Space cannot restart the failed LIVE game

Location: `AGP/src/games/squash/live.rs:50..65,94..116`;
real host `host/cases/squash_game.rs:1230..1286` maps Space into this state
machine and only processes `Some(RetryPressed)` to reload the scene.

Frozen `Main.gd` handles ui_accept while Retry is visible; manifest E6 and the
visible Retry label explicitly promise **Space or Enter**. In the bound probe,
Space press with dead-state action yields `effect=None`, `retry=false`, and
`take_retry_edge=false`. Enter positive yields `Some(RetryPressed)`. There is
no later Space retry handler in the host. This is a deterministic input-path
defect, independent of the desktop focus issues described above.

The old unconditional auto-jump defect is fixed; that specific control remains
closed. The repair did not cover the sibling dead/retry use of the same key.
AC-04; T-06/07. Cover living jump edge, held/release, dead Space and Enter retry,
retry visibility, and the first input after reload through the actual host
mapping. The expected scene reset is the existing sample behavior.

### F829-R4-03 — P2: equivalent globally rotated encounters change death outcome

Location: `AGP/src/games/squash/sim.rs:200..230`, wired at541.
The detector first transforms the mob center into the player's local frame,
then applies `rotation_y(-mob_yaw)` as if the mob yaw were still measured in
that frame. Player-facing rotation is counted inconsistently. This is an
exact coordinate-frame issue, not a Jolt integrator/precision tolerance.

**Bound input:** player at origin, facing0, mob position `(-2,.5,-.7)`, yaw0;
rotate both bodies and all relative positions90° around Y: player facingπ/2,
mob `(-.7,.5,2)`, yawπ/2. Both body-sphere/box contact tests are negative.
**Expected:** identical detector classification under this rigid rotation.
**Observed:** helper false→true. The preserved probe then enters the **actual
Runtime/SquashDriver tick** with the same relative setup translated to resting
player height, negligible forward velocities establishing each mob yaw:
`dead_base=false`, `dead_rotated=true`.

The legitimate no-rotation and original detector-only tests pass; the problem
is the previously untested joint player/mob orientation. R1-F03's original
example is closed but its full orientation invariant is not.
AC-04; T-06/07. Use consistent relative frames and verify multiple facing/mob
yaw pairs, global rotations, body-free detector hits/misses and valid top hits.
Do not tune the test coordinates or regenerate the golden trace to hide this
source-coordinate error.

### F829-R4-04 — P2: shared scene snapshots omit the newly supported rotation

Location: `AGP/src/core/scene.rs:219..241`. Transform now includes quaternion
rotation and render/replay uses `to_matrix`, but snapshot serializes only
translation/scale/color/resource. Its documented "observationally identical"
claim is therefore false for supported3D state.

**Bound public API:** spawn an identity-transform object, snapshot, submit only
rotationY=180° (`Quaternion(0,1,0,0)`), snapshot again. **Expected:** different
render transform produces distinguishable snapshots. **Observed:** equal
snapshots, different matrices. Legitimate unchanged-state equality still holds;
the issue is the omitted supported state, not session-independent ID comparison.

This weakens state/hash/change-log acceptance after the F11 extension; earlier
2D-only tests do not guard the new field. AC-02/05/06; T-03/06/07. Include the
effective orientation in canonical state evidence and recheck replay/reset,
rotation-only edits and legitimate equivalent/unchanged-state controls.

### F829-R4-05 — P1: invisible final Iced composition still passes

Location: `AGP/src/host/cases/iced_shared_device.rs:58..80,197..209` and
`host/iced_spike.rs:363,662..676`. Composite verification now counts actual
draw calls; it does not observe the resulting composed pixels/content.

**Actual mutation and controls:** the isolated-title original-shader positive
exits0; the fully transparent final-fragment negative also exits0. In the
negative report: `exit=pass`, real presses1, `all_gates_behaved=true`,
`composite.ok=true`, draw counts **1→19→22** for both views. Offscreen captures
and all counters pass while final graphics contribute no visible pixels.
This executes the normal compositor draw pipeline and the unmodified official
gate; it does not merely alter a printed report or sabotage an artificial flag.

R1-F05's exact skip-draw example is now caught, but the accepted **visible
composition/alpha/clipping** invariant is still open. GDI capture difficulty
does not establish that a counter proves this invariant or remove the agreed
V-06/V-08 requirement. AC-05/06/07; T-07/08/09. Add an independent observable
of the final composed result and a matching positive/negative content oracle.
It must reject skipped draws, fully transparent/blank result, wrong target or
clip placement while the legitimate click/resize/DPI case remains green.

## R1/R2/R3 evidence reconciliation

| Prior issue | R4 disposition |
|---|---|
| R1-01 auto-jump, R1-02 redraw timing | original examples closed: real input-edge/elapsed-time code and tests; R4-02 is a missing retry sibling |
| R1-03 source-state repair | origin-target example closed; orientation invariant partial via R4-03 |
| R1-04 cache key/ID lifetime | closed within previous scope: binding/freeing code and all three real-GPU tests pass |
| R1-05 final composition | skip-draw example closed; full invariant **partial/open**, R4-05 |
| R1-06 lifetime oracle | retain previous scoped witness/cancellation/sampling closure, current positive passes; this is not a claim of exhaustive OS/GPU leak proof |
| R1-07 independent source/content oracle | **partial/open**: source inventory/payload checks and frame integrity improved, but implementation-derived frames accept the source-forbidden post-death player image (R4-01) |
| R1-08 metrics | scoped instrumentation/raw-series/fingerprints closure retained; current positive passes; unchanged accounting controls from R2/R3 are historical evidence |
| R1-09 patch boundary | closed: stock iced_widget, copied tree absent |
| R1-10 health | scoped owned-source health repaired,90/no warning/fmt0; stale current-acceptance statements must be updated after these findings |
| R1-11 shared3D | actual deltas exist; whole-state/type invariants **partial**, R4-01/04 |
| R2-01..05 | targeted fixes retained; new findings are not a rollback to old headless wiring, stale release-binary, patch wording or count defects |
| R3-01 input-wait timeout debt | still present; two inconclusive attempts above are not used as rendering failures. Original/isolated positives did run successfully this round |

### The proposed substitute source oracle is not independently sufficient

The r1 contract requires source-based behavior/content expectations frozen by
T01/T02, and says implementation must not redefine its own oracle. R2/R3
accepted implementation-captured goldens and deferred character-level engine
comparison. A reviewer decision cannot make an existing source requirement
optional. Here the concrete counterexample removes any ambiguity: the frozen
retry frame contains a player after its ID is destroyed, and the official
per-pixel comparison accepts it. Thus the substitute did not establish the
already-required rule even if its integrity and corruption controls pass.

Godot METHOD.md also explicitly records missing imported characters/audio and
nonmatching gameplay outcomes. Those outcomes are not classified here as a
separate physics regression: RNG streams and integrators differ. That incomplete
run simply cannot prove the absent character-level/render-state expectation.
Close the concrete source invariants with independently established assertions/
reference evidence, or obtain a real approved contract revision; do not defer
the existing requirement as debt solely on the strength of R2/R3 labels. This
is continued R1-F07, not a newly imposed whole-engine compatibility requirement.

## AC/task/Spec disposition

| AC | Result at this SHA |
|---|---|
| AC-01 | partial: independent boundary/assets valid; source oracle completeness remains R1-F07 (T01/09) |
| AC-02 | partial: generic foundation and cache valid; snapshot/type integration defects R4-01/04 (T03/06) |
| AC-03 | scoped pass: current2D tests/offscreen runtime green; same-SHA windowed evidence from R3 retained, not claimed newly rerun |
| AC-04 | fail: actual source semantics violated by projection, retry and orientation (R4-01/02/03; T06/07) |
| AC-05 | partial/fail: improved lifecycle/stale controls; scene-state and compositor discrimination still fail (T03/06/07/08) |
| AC-06 | partial: CPU/raw/fingerprint method repaired; snapshots/oracle/composite can false-green (T07/09) |
| AC-07 | fail: real positive green, invisible final composition also green (T08) |
| AC-08 | partial: I1–I3 pending and IH owners/gates appropriately retained; current verified-fact/SD statements need findings reconciled (T09) |
| AC-09 | fail: independent review performed, open in-contract findings prevent final PASS (T10) |
| AC-10 | pending merge-owned, not executed |

SD-01 project entries/boundaries remain appropriate proposals, source-fidelity
assertions need reconciliation. SD-02 supported3D state/replay/verifier facts
are not fully verified at this commit. SD-03 visible composite/alpha/clip/exit
claims exceed the current gate's discrimination. Targets remain the three
new `docs/specs/auto-graphics/...` files; supersedes[], GOAL-008 retained.
Frozen SD copies are **not approved for deposition** this round.

## Routing and required diagnosis

Return to **work in the same tree**. Reopen T01,T03,T06,T07,T08,T09 and T10;
preserve T02,T04,T05's completed work/evidence. Status executing,
**current_step3 / total_steps11**, T11 pending. No semantic Plan revision made.

This follows multiple failed final reviews. The prior E15 diagnosis addresses
render-mode wiring, but the new gaps show missing **state/orientation/input**
coverage and testing an old mutation rather than the complete invariant.
Before re-handoff, extend the finding-closure diagnosis with alive→dead→empty
and leading-object deletion, role identity after reuse, player/mob joint yaw,
rotation-only scene state, Space/Enter while alive/dead, and drawn-but-invisible
final composition. Bind each to actual producer/consumer/assertion and legitimate
controls; independently establish the content expectation before re-freezing.

Keep pending merge-owned gates explicit: final acceptance of repaired frozen
code; applicable independent-workspace tests/probes and real consumer relocation;
migration/write-surface handoff authorizing landing; canonical Specs/derived
ledger reconciliation; archive and guarded cleanup. No root tiers apply while
only AGP is touched. No implementation fix, merge, archive, branch removal,
canonical Spec/ledger publication or external message was performed in review.
