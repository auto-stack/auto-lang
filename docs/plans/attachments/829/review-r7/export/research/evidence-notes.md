# PLAN-829 T-01 Evidence Notes — corrections / additions to Plan §4.3

These notes record gaps between the Plan's §4.3 observations (and the planning
preflight's cautions) and what the frozen sources actually contain. They are
evidence only: **sample targets are unchanged**. Everything below was parsed
from the frozen sources listed in `research/sample-manifest.md` on 2026-10-20.

## E1. Music playback path — preflight caution resolved (AG-3D-01)

- Preflight caution: "README 的音频版权说明不等于播放证据".
- Actual finding: `MusicPlayer.tscn` (an `AudioStreamPlayer`, `autoplay = true`,
  looping OGG) IS instanced — as an **autoload singleton** via
  `project.godot [autoload] MusicPlayer="*res://MusicPlayer.tscn"`. It does not
  appear as a node inside `Main.tscn`, which is why a scene-only read misses
  it. Conclusion: the music genuinely plays from session start; it is the
  **entire** audio denominator for the sample (no SFX of any kind exist in the
  frozen tree).

## E2. Animations live in the scenes, not the GLBs (AG-3D-01)

- Plan §4.3 listed "脚本/动画" as T-01 inventory items; a naive reading might
  expect animations inside `player.glb`/`mob.glb`.
- Actual finding: both GLBs contain **zero animations and zero textures**
  (verified by parsing the GLB JSON chunk). The only animation is the inline
  1.2 s looping `float` Animation sub-resource defined identically in
  `Player.tscn` and `Mob.tscn` (position bob + pitch wobble on
  `Pivot/Character`), with `autoplay = "float"`. Player scales it ×4 while
  moving; mob scales it by `speed/min_speed`. The AGP driver must replicate
  this scene-level animation, not look for it in the models.

## E3. Mob physics — mobs collide with nothing (AG-3D-01)

- Not stated in the Plan: `Mob.tscn` has `collision_layer = 2` (enemies) and
  `collision_mask = 2147483648` (no defined named layer), so mobs do not
  physically collide with the world, the player body, or each other. All
  player-mob interaction is mediated by (a) the player's slide-collision
  squash check (top contact, normal·UP > 0.1) and (b) the player's
  `MobDetector` Area3D (mask = layer 2) for death. Mobs are freed on
  screen-exit via `VisibleOnScreenNotifier3D`.

## E4. Player-mob physical contact exists (AG-3D-01)

- Counterpart to E3, also not in the Plan: the player's
  `collision_mask = 2147483654` includes layer 2 (enemies), so the player body
  DOES slide-collide with mobs — this is what feeds the squash check. A
  faithful driver needs both channels (physical slide collision + area
  detection), not just one.

## E5. Physics timing (AG-3D-01)

- `project.godot` pins `physics_ticks_per_second = 120`,
  `physics_engine = "Jolt Physics"`, `physics_interpolation = true`, MSAA 3D
  quality = 2 (4×), directional soft-shadow filter quality = 3. The 120 Hz
  physics rate is frozen into the oracle (manifest §2.6). The engine being
  Jolt means exact integrator parity is not expected — tolerances are the
  contract, not step-for-step equality.

## E6. Retry uses built-in `ui_accept` (AG-3D-01)

- Retry input is `event.is_action_pressed("ui_accept")`, and `ui_accept` is
  NOT bound in the frozen `project.godot` — it is Godot's built-in default
  (Space / Enter). The Retry label text confirms: "Press Space or Enter to
  retry". AGP's frozen keyboard profile maps this to Space and Enter.

## E7. Breakout stepping module confirmed inert (AG-2D-01)

- `breakout.rs` compiles with `mod stepping;` (a UI for the
  `bevy_debug_stepping` cargo feature). At the frozen commit the plugin body
  returns early when the feature is off (`if cfg!(not(feature =
  "bevy_debug_stepping")) { return; }`), so gameplay is unaffected and no
  stepping UI exists in a default build. Consistent with Plan V-03 ("Bevy
  stepping 工具不在玩法 profile"); now source-verified. `stepping.rs` is
  copied into `testdata/breakout/` because `breakout.rs` references it.

## E8. Breakout asset/font denominator (AG-2D-01)

- The ONLY asset consumed by the frozen breakout sample is
  `assets/sounds/breakout_collision.ogg` (the single `asset_server.load` call,
  line 179). No font files, no textures, no spritesheets. Bevy's default font
  renders the scoreboard (font size 33 px, "Score: " + number). The collision
  sound has no entry in the frozen Bevy `CREDITS.md` (license gap recorded in
  the manifest §3; upstream introduced it in PR #4331 without a note).

## E9. Squash texture denominator is empty (AG-3D-01)

- Plan §4.3 asked to verify assets "都在播放/使用". Finding: the sample has
  **no textures at all**; the two GLBs use untextured flat-color embedded
  materials, and the five `art/*.tres` material files are unreferenced
  leftovers (grep-verified against every .tscn/.gd/.import in the frozen
  tree). Runtime-relevant assets = 2 GLBs + 1 music OGG + 1 font (+ its
  license). Everything else in the directory is editor metadata, source art
  (.blend), icon, or screenshot.

## E10. Renderer profile (AG-3D-01)

- README states "Renderer: Forward+"; `Main.gd` contains a `gl_compatibility`
  workaround branch (PCF13 shadow filter + duplicated dimmed light) that is a
  no-op on the Forward+ profile. AGP (wgpu) should treat the Forward+ visuals
  as the target and record the gl_compatibility branch as a known platform
  difference, not replicate it.

## E11. Breakout HUD font (AG-2D-01)

- The frozen `breakout.rs` loads NO font asset; the scoreboard uses Bevy's
  built-in default font. At the frozen commit that default is
  `crates/bevy_text/src/FiraMono-subset.ttf` (verified in the frozen tree:
  `bevy_text` embeds it via `include_bytes!` in `font.rs`).
- Closest documented choice for AGP: `assets/fonts/FiraMono-Medium.ttf` from
  the SAME frozen Bevy tree (`assets/fonts/`, SIL OFL 1.1 per
  `assets/fonts/FiraMono-LICENSE`, © Mozilla Foundation/Telefonica) — same
  family as the sample's default, full glyph coverage (the embedded subset
  only covers a limited charset). Copied to
  `assets/breakout/fonts/` with its license; sha256 recorded in
  `testdata/breakout/FONTS.sha256`. Rasterization differs from Bevy's (AGP:
  fontdue atlas; Bevy: cosmic-text/parley) — glyph shapes are not
  oracle-compared, only presence/position/count of HUD text (9 glyph quads
  for "Score: N" at 33 px, top-left 5 px padding, two-tone colors per the
  frozen source).

## E12. Review-r1 semantics repairs (F01/F02/F03/F11, 2026-10-21)

Independent review r1 (`docs/plans/attachments/829/review-r1/review.md`)
found four contract violations; all were repaired against the frozen source
(the oracle was NOT weakened):

- **Spawn aim (F03)**: mobs now aim at the player's ACTUAL position at spawn
  time (Main.gd line 36), not the literal origin. This legitimately changed
  gameplay semantics → the golden trace was REGENERATED by the independent
  Python reference (same frozen constants; seed re-frozen to **0x1B** after
  a documented seed/factor sweep reproduced the scripted scenario: 3
  squashes then exactly 1 death; 22 spawns, 45 events). Header provenance
  updated. The Rust sim replays the new trace bit-exactly (max pos diff 0.0,
  45/45 events, per-tick vy now also compared).
- **MobDetector channel (F03)**: the five frozen cylinders (Player.tscn
  lines 63–83) are implemented as a second fatal-contact channel, rotating
  with the player's facing yaw; overlap is fatal even without body contact
  (review probe geometry covered by test). Body top-landing squash channel
  unchanged (normal·UP > 0.1).
- **Facing/animation (F03)**: player facing yaw tracks movement
  (Basis.looking_at), animation speed 4 moving / 1 idle as a clock warp,
  parent pitch rotation.x = PI/6 · vy / jump_impulse (Player.gd line 66),
  mob bob phase = age since spawn.
- **Live input (F01)**: interactive jump is a real press edge
  (`is_action_just_pressed`, Player.gd line 39) via `LiveSquashInput`; no
  more auto-jump, held-Space fires once until re-press, dead gates jump.
- **Live timing (F02)**: both live hosts advance fixed ticks from elapsed
  wall time via `TickClock::feed_capped` (catch-up cap 4/8 ticks, debt
  dropped under overload — documented policy). Canonical replay paths
  untouched (bit-exact).
- **Shared 3D contract (F11)**: squash now submits scene deltas to the
  G-base runtime (spawn/animate/squash/despawn); entities have session
  object IDs; rendering projects from the shared scene. An independent
  consumer replays the ChangeLog, reconstructs the world, edits a mob via
  its shared ID, and gets stale IDs rejected across restart
  (`f11_shared_contract_reconstructs_and_edits_3d_scene`).

Residual documented approximations (unchanged): single-pass move_and_slide;
WorldBoundaryShape inward planes; cylinder-vs-OBB tested as y-interval +
exact 2D circle-vs-rotated-rect (the frozen geometry is exact; Jolt's
continuous Area3D signal timing is approximated discretely at 120 Hz).

## E13. Review-r1 batch-3 repairs (F05/F07/F09/F10, 2026-10-21)

- **F09 (patch model)**: the production PLAN-043 iced_widget patch is NOT
  necessary for AGP's minimal static spike — verified by de-vendoring and
  running the canonical iced case green on STOCK crates.io iced_widget
  0.14.2 (layout, views, resize, DPI, click all correct). The vendored
  `patches/` tree and the `[patch.crates-io]` table were REMOVED; AGP now
  has ZERO path dependencies. Production retains the patch — the semantic
  difference is recorded, not hidden.
- **F05 (final-composite gate)**: GDI window capture (PrintWindow
  CopyFromScreen) HANGS on the presenting wgpu window from a non-owner
  process in this environment (attempted, recorded). The composite
  evidence is therefore in-band: blit-draw counters incremented ONLY
  adjacent to the real composite draw call, sampled at each key moment;
  the review's blank-blit mutation turns the gate RED (recorded in
  scratch/829/iced-sabotage-blank/). A foreign device cannot be detected
  at runtime (wgpu global resource ids, batch-2 finding), so the
  foreign-device invariant is structural: the permanent audit test
  `f05_render_path_has_no_second_device`.
- **F07 (source oracle)**: manifest contract reconciled in place (seed
  0x1B, spawn every 60 ticks — dated correction); runner now compares FULL
  event payloads (kind+tick+index+score) and FULL spawn tuples
  (position+yaw+speed+velocity+tick); content oracle = frozen golden
  keyframes (testdata/golden_frames/, sha256 + tolerances) compared
  per-pixel in both game gates. ORIGINAL-ENGINE ground truth: Godot
  4.4.1-stable downloaded and run against the frozen project with the
  frozen action stream — trajectory/semantics baseline captured (see
  testdata/golden_frames/godot-engine-reference/METHOD.md; characters not
  renderable due to the headless import-scan blocker; R2 decision item
  recorded there).
- **F10 (health)**: cargo fmt clean; zero warnings across lib+tests;
  docs reconciled.
- **Environment facts**: the iced OS click is flaky under the agent
  session's focus stealing; the input evidence records WHICH path
  delivered the press ("os_focus" real foreground click vs "posted_queue"
  WM_LBUTTON through winit's real message pipeline, retried until
  observed).

## E14. iced_spike live "freeze" report — NOT reproduced (2026-10-21)

User reported the LIVE `iced_spike` window freezing after clicking
"Step graphics". 7 instrumented live-window runs (posted clicks, real
foreground clicks, rapid bursts, two-instance control, resize) with
`AGP_TRACE`-gated eprintln traces (since reverted) all show the event
loop alive at 30 ticks/s + 60 renders/s through and after every click
pattern; the user later confirmed the extra presses seen in the traces
were their own manual clicks and that the app no longer freezes.

- **Lock-graph audit**: the only nested lock in `iced_spike.rs` is
  `GraphicsProgram::draw` (GRAPHICS→SHARED_EVIDENCE, `:264–265`); every
  other site holds at most one mutex; no SHARED_EVIDENCE→GRAPHICS
  ordering exists anywhere → acyclic, no deadlock is possible in the
  click path. The live click handler touches exactly one lock.
- **Observed stalls**: a single ~500ms stall sat between the last blit of
  frame N and the next tick (wgpu present / winit event wait), fully
  recovering with a tick-drain burst; the host also throttles background
  processes erratically (trace clock 87s over 238s wall).
- **Conclusion**: most consistent with environment-level CPU/GPU
  starvation during the original report window (heavy parallel build
  load), not an app block. No code change made; the frozen baseline
  `1fc3f6116` stands. Traces archived under `scratch/829/live-freeze/`.

## E15. Review-r2 repairs (F829-R2-01..05) + two-failure diagnosis (2026-10-21)

R2 (independent, frozen commit `95ccfcc5c`) returned needs_fix on one P2
gate-wiring defect introduced by the r1 repair + four P3 fact/robustness
nits. All repaired in the r3 batch:

- **F829-R2-01 (P2)**: the r1 F07 repair wired the per-pixel golden-frame
  oracle only into the WINDOWED capture path
  (`BreakoutApp::capture_keyframe`, `src/host/cases/breakout.rs:820`),
  while the pass predicate requires non-empty `golden_frame_checks`
  unconditionally. The offscreen path (`capture_keyframes_offscreen`,
  used by `--mode headless` AND the windowed run's offscreen fallback)
  wrote keyframes with no oracle → `ag2d-breakout --mode headless`
  deterministically exited 1 with every gameplay fact green (headless
  frames are bit-identical to the goldens — pure wiring). Fixed by
  running `frame_oracle::compare_golden_frame` in the offscreen path with
  identical semantics to the windowed path. Verified: breakout headless
  AND windowed exit 0.
- **F829-R2-02 (P3)**: `independent-consumer`'s A-side reused whatever
  `target/release/graphics-probe.exe` the tree last built — a false-red on
  a fresh frozen commit (`fingerprints.agp_commit 1fc3f61164b0 vs
  95ccfcc5c18a`, a docs-only delta). The A-side reference binary is now
  built in-case from the original tree into an isolated
  `CARGO_TARGET_DIR` (`scratch/829/consumer-a-target`, incremental across
  runs) before the reference run; the isolated dir both removes reliance on
  a possibly-stale `target/release` binary and avoids the Windows file lock
  on a running `target/release/graphics-probe.exe` (first r3 attempt rebuilt
  in-place and hit os error 5 — recorded). The "self-contained" comment now
  states this honestly.
- **F829-R2-03 (P3)**: three stale "vendored iced_widget patch"
  statements removed (`integration-handoff.md` IH-05,
  `spec-delta/auto-graphics.project.md` SD-01,
  `spec-delta/auto-graphics.integration-contract.md` SD-03); each now
  states the F09 reality: stock iced_widget 0.14.2 from crates.io, zero
  path-sourced packages, production retains the PLAN-043 patch. A
  fourth stale instance (same class) was found during r3 in the
  `independent-consumer` report notes (consumer.rs) and corrected too.
- **F829-R2-04 (P3)**: `sample-manifest.md` §2.6 "one mob every 30
  physics ticks" reconciled to 60 with a dated correction note.
- **F829-R2-05 (P3)**: stale counts fixed to actuals — squash contract
  header and the result.md AC-04 row now say 45 events (22 spawns, 3
  squashes, 1 death, 19 despawns; verified against
  `testdata/squash/golden_squash_trace.json`), and the intermediate
  "85 tests (lib 23 … iced seam 4)" line now reads the measured 90.

**Two-failure diagnosis (WHY R2-01 slipped two stages)**:

1. *Stage 1 (work)*: F07 verification ran the oracle only in the mode it
   was wired for — windowed — because the r1 repair's own verification
   replayed R1's counterexample (corrupt a golden frame → case must go
   red) through the windowed path where the wiring existed. The
   windowed run passed, so the repair was declared done.
2. *Stage 2 (R1 review)*: R1's sibling-path demand for the F07 oracle was
   tracked at the *predicate* level ("the pass predicate requires
   golden-frame checks and corruption discriminates"), never decomposed
   per render mode. The predicate exists, the corruption replay fires,
   verdict: closed. Nobody asked "does the predicate's evidence producer
   run in EVERY mode the case advertises (`--mode auto|window|headless`)?"
   — the case's usage string advertised headless, R1's own replay
   methodology ran breakout headless, and the plan allows per-case
   headless modes; the offscreen producer was never on the checklist.

*Detection adjustment (now applied)*: work verification must run EVERY
game/verifier probe case in BOTH windowed and headless modes before
handoff (where the case supports the mode), and oracle gates are checked
per mode — not only at the predicate level. The r3 verification matrix
below applies this rule: `ag2d-breakout` and `ag3d-squash` each exit 0 in
windowed AND headless; the remaining cases run in their canonical modes.

## E16. Review-r4 repairs (F829-R4-01..05) + diagnosis extension (2026-10-21)

R4 (independent re-review, frozen commit `077de3b66`) returned needs_fix on
five in-contract invariant gaps. All repaired at the invariant level — no
oracle weakening, no coordinate tuning, no golden regeneration to hide
errors:

### F829-R4-01 — player role identity unstable through death (P2)

- **Root cause**: `project_instances` classified the FIRST ALIVE scene
  object as Player (`sim.rs` — kind by the mutable vector head). After the
  player's Destroy, a surviving mob inherited player meshes; the
  classification changed again as leading mobs despawned. Introduced by the
  F11 shared-scene repair; the F11 test stopped at tick 600 (player alive)
  and never observed the deletion state.
- **Fix**: `Scene` now records the session's FIRST-SPAWNED object id
  (`first_spawned`, set once on the first Spawn, survives destruction and
  index/generation reuse); `project_instances` classifies by that stable
  identity. An independent consumer replaying the ChangeLog derives the
  same rule (first Spawn of the session). The driver never respawns the
  player within a session (retry = full session rebuild), so the anchor is
  exactly the player.
- **Permanent controls** (`r4_player_role_identity_stable_through_death_and_reuse`):
  before-death classification (bound probe: player id = (index0,generation0),
  first spawn); destroyed identity invalidated in the scene; after-death
  with live mobs — ALL remaining project as Mob; leading-mob deletion
  sequence (per-id kinds stable as the population shrinks); empty scene
  (nothing projects, anchor inert); restart (fresh session, new anchor);
  independent-log-replay classification (same per-id kinds, session-exempt
  identity comparison).
- **Golden disposition** (`retry_overlay.png` RE-FROZEN): the old golden
  contained the post-death player image (a mob rendered with player meshes)
  and the per-pixel gate had accepted it (changed region = one 100x66 bbox
  = 0.66% of pixels, 3563 px, mean diff 0.995/255 — inside the manifest
  tolerances). Source re-derivation: `Player.gd` lines 68-70 —
  `die() = hit.emit() + queue_free()` — NO player image may exist after
  death; verified the changed bbox is exactly the misclassified mob
  (dominant orange player pixels replaced by dark-blue mob pixels).
  `squash_moment` and `final_frame` verified BIT-IDENTICAL before/after the
  fix and were NOT re-frozen. This is a source-derived expectation repair,
  NOT an oracle redefinition: tolerances, the sha256 integrity discipline
  and the corruption-discrimination control are unchanged (recorded in
  `testdata/golden_frames/manifest.json` provenance, dated).

### F829-R4-02 — Space cannot restart the failed LIVE game (P2)

- **Root cause**: the F01 repair armed Space's press edge for jumping only;
  the dead/retry sibling of the same key was never wired. Enter fired
  `RetryPressed` unconditionally (host-gated); Space in dead state produced
  `effect=None` — violating the frozen promise (Main.gd `_unhandled_input`:
  ui_accept while Retry visible = reload; manifest E6 "Space or Enter").
- **Fix**: `LiveSquashInput::key_event` now takes the driver's dead-state.
  Space press edge while dead = retry edge + effect, and NO jump edge is
  armed (a stale edge can never fire a jump on the first tick after the
  reload; `space_held` updates before the effect return). Enter now fires
  the retry only while dead (machine-level, not merely host-gated). The
  host passes `driver.dead` through the real winit keyboard path.
- **Permanent controls** (`r4_dead_space_fires_retry_not_jump`,
  `r4_dead_enter_fires_retry`, `r4_living_space_is_jump_edge_only`,
  `r4_dead_space_and_enter_retry_through_host_mapping`): dead Space retry,
  dead Enter retry, living Space = jump edge only (no retry effect),
  living Enter = nothing, held/release semantics across death+reload,
  retry-visibility gating (alive / not-visible = no reload), first input
  after reload.

### F829-R4-03 — equivalent globally rotated encounters change death outcome (P2)

- **Root cause**: `detector_fatal` rotated a PLAYER-LOCAL offset by the
  mob's WORLD yaw (`rotation_y(-mob_yaw)` on a player-local vector) —
  mixing frames. Only relative geometry may matter; the correct rotation is
  `rotation_y(player_facing_yaw - mob_yaw)`.
- **Fix**: one-line frame correction with the full derivation recorded in
  a comment.
- **Permanent controls** (`r4_detector_classification_rigid_rotation_invariant`):
  the review's bound probe (base vs 90-degree-rotated, both
  body-contact-negative) plus four encounter classes (bound-probe miss,
  frozen F03 hit, 2D corner hit, y-channel miss) x five global rotations
  (0, pi/2, pi, -1.1, 2.5) — classification identical under rigid rotation
  and under absolute re-orientation; the pre-existing F03 assertions
  (including the facing-pi negative, re-derived from source geometry) stay
  green. Coordinates reuse the frozen probe geometry — no tuning.
- **Golden disposition**: the canonical trace replay stays BIT-EXACT after
  the fix (`golden_trace_replay_matches_reference` green) — the frozen
  scenario's death occurs at facing-zero geometry, so no trace regeneration
  was needed. Said so with evidence instead of regenerating.

### F829-R4-04 — shared scene snapshots omit rotation (P2)

- **Root cause**: `Transform` gained quaternion rotation (F11) and the
  renderer consumes `to_matrix`, but the canonical snapshot serialized only
  translation/scale/color/resource — rotation-only states produced equal
  snapshots while their matrices differed.
- **Fix**: snapshot now includes the rotation quaternion, sign-
  canonicalized (q ~ -q is the same rotation: negate when the first
  non-zero component is negative; -0.0 normalized) so matrix-equivalent
  orientations snapshot equal.
- **Permanent controls** (`r4_snapshot_distinguishes_rotation_only_changes`):
  the bound probe (rotationY=180 degrees on an identity object =
  distinguishable snapshot AND different matrix), unchanged-state equality,
  sign-flipped equivalent equality, identity-restore equality. All snapshot
  consumers are equality-based (shared-runtime replay equality, breakout
  report hash, prototype contract) — no frozen hash constants existed, so
  the format extension required no assertion re-basing; shared-runtime
  exit 0 re-verified after the change.

### F829-R4-05 — invisible final Iced composition still passes (P1)

- **Root cause**: the F05 composite gate counts real draw calls — it never
  observes composed PIXELS. The review's fully-transparent final-fragment
  mutation (offscreen views intact, counters advancing 1 to 19 to 22)
  passed with `composite.ok=true`.
- **Fix**: per-moment composed-content gate. iced 0.14 DOES expose an
  in-compositor readback — `window::screenshot(id)` re-runs the FULL UI
  draw (background, controls, both view blits with clipping) through the
  same engine and reads the pixels back — the identical composition the
  window presents, not a re-render of our own textures. At each canonical
  moment (initial / after_button / resized) the app captures a screenshot
  and analyzes it: (1) non-blank (>2% pixels differ from the flat window
  background), (2) UI controls present (solid non-bg fill in the control
  band), (3) 2D view present at the layout-derived rect (source-anchored
  clear color (20,23,31) +/-16, at least 20%), (4) 3D view present
  (source-anchored sky clear color [0.64625,0.65575,0.67075] visible at
  least 3% — survives the camera-aspect change at 860x700, measured 6.6% —
  AND at least 4 quantized colors in the rect), (5) clip: no 2D-view
  background outside its rect (gap band, padding columns, control band all
  <1%). All moments must pass; a missing moment is a failure.
- **Integration constraint found in r4**: iced executes the screenshot's
  GPU readback SYNCHRONOUSLY inside the event loop (measured ~40-60 s under
  this host's erratic throttling; environment E14 fact). The click
  fallbacks' retry window raced the blocked loop. Fixed by phase ordering:
  click dispatch happens in a phase AFTER the initial screenshot, never
  concurrently. Documented in `canonical_step`.
- **Permanent negative control**: `AGP_SABOTAGE_ICED=transparent_blit`
  forces the blit fragment to `vec4(0,0,0,0)` (ALPHA_BLENDING = no visible
  contribution; the exact R4 mutation). Demonstrated RED twice: the F05
  counter gate stays green (draws 1 to 980 to 985 advancing — the review's
  exact scenario) while the content gate reports `content ok=false`
  (f_v2=0.000, f_sky=0.000, v3_colors=0, non-blank 0.6%) at every moment;
  case exit 1 (`scratch/829/r4/iced-transparent-neg2/`). The positive stays
  green (exit 0, all three moments ok, `scratch/829/r4/iced-positive5/`).

### Diagnosis extension (multiple failed final reviews)

Each missed invariant, why earlier checks missed it, and the binding now in
place:

| Missed invariant | Why it slipped | Binding now applied |
|---|---|---|
| Alive-to-dead-to-empty projection; role identity after reuse (R4-01) | The F11 test asserted the projection only while the player was ALIVE (tick 600); "entity type preserved" was tracked as a data-model claim, never exercised through the deletion sequence | Permanent death-sequence test (all five states); golden re-frozen from source-derived expectation |
| Player/mob JOINT yaw under global rotation (R4-03) | R1's detector counterexample used facing=0 geometry, where the two frames coincide; the orientation INVARIANT (rigid-rotation equivalence) was never stated | Rotation-invariance test: 4 encounter classes x 5 global rotations |
| Rotation-only scene state in canonical snapshots (R4-04) | Snapshot field coverage lagged the F11 Transform extension; earlier 2D-only snapshots could not express rotation | Rotation in snapshot + sign canonicalization; bound-probe test |
| Space/Enter retry while dead (R4-02) | F01 fixed the jump press-edge and tested alive semantics; the state machine's dead-side siblings of the SAME keys were untested | Dead-side retry tests for both keys + visibility gating + first-input-after-reload |
| Drawn-but-invisible final composition (R4-05) | Draw-call counting was accepted as a substitute for pixel observation (GDI capture hangs made pixels "impossible" — until the in-compositor readback was found) | Per-moment composed-pixel gate + permanent transparent-blit negative control (env sabotage, demonstrated red) |

Detection adjustment (now the standing rule): every verifier gate must name
the INVARIANT it observes (state sequence, coordinate frames, field
coverage, key siblings, pixels) — not the mechanism that usually satisfies
it; every repair replay must exercise the full invariant (all states / both
keys / all modes / the rendered result), not merely the counterexample that
motivated it.

## E17. Review-r6 repairs (F829-R6-01..03) + diagnosis extension (2026-10-21)

R6 (independent re-review, frozen commit `b84c6d91e`) returned needs_fix on
three findings, repaired at the invariant level — no coordinate tuning, no
oracle weakening, no hiding errors behind regenerated goldens.

### F829-R6-03 (P1) — required golden manifest missing from the commit

- **Root cause**: `testdata/golden_frames/manifest.json` is a REQUIRED
  runtime input of `host/cases/frame_oracle.rs`, but the repo-wide
  `*.json` gitignore (meant for throwaway report.json files) excluded it.
  Every green content-gate run used a file OUTSIDE the reviewed revision;
  R6 proved a `git archive HEAD` export reports `compare=None` — the games'
  acceptance condition is unsatisfiable from a committed-only delivery.
  R5's clean-tree check missed it because `git status` clean ≠ committed
  closure (ignored files are invisible to status).
- **Fix**: nested `testdata/golden_frames/.gitignore` negates the rule for
  exactly this file (`!manifest.json`) — the manifest is now versioned
  while the report.json convention stays. Force-added and verified
  `git check-ignore` now rejects it.
- **Delivery-closure audit (permanent)**: `tests/delivery_closure.rs` binds
  the full inventory of runtime-required inputs the probe cases and gates
  read from the tree — golden manifest + SHA256SUMS + every
  manifest-declared frame, both frozen traces, both HUD fonts, both audio
  assets, both GLB models, and the assets/testdata/FONTS integrity
  receipts — asserting each EXISTS and (inside a git checkout) is TRACKED
  and NOT ignored; a second test binds that the manifest declares only
  frames that exist. Negative-control paths (`__missing__.*`) are
  deliberately excluded — they must not exist.
- **Committed-only export replay**: recorded in result.md — both games run
  green from a `git archive HEAD` fixture containing only committed files.

### F829-R6-01 (P2) — source spawn height is local y=0, not y=1

- **Root cause (local/global coordinate misread)**: the frozen SpawnPath
  sits at transform y=1 (Main.tscn line 128), and every AGP layer read
  that as the spawn height. But `Main.gd:37` passes
  `mob_spawn_location.position` — the PathFollow3D's LOCAL position
  (SpawnLocation local y=0, line 132; curve points y=0) — and `Mob.gd`
  adds the mob directly under Main, so SpawnPath's y=1 never reaches the
  mob. The misread propagated through implementation (`sim.rs` SPAWN_Y),
  the independent Python generator, the frozen golden trace, and
  `research/3d-feasibility.md` ("matching the frozen source exactly").
- **Evidence**: R6 ran the REAL Godot 4.4.1 fixture with the unchanged
  frozen Main.gd/Mob.gd (minimal source-emitter test, four seeds):
  `path_y=1 follower_local_y=0 follower_global_y=1 actual_spawned_mob_y=0`
  in every trial. AGP emitted its first spawn at y=1 — the global-position
  value is exactly the misread control.
- **Fix (implementation + research + oracle TOGETHER)**: `SPAWN_Y = 0.0`
  in `sim.rs` with the source derivation; the generator's `SPAWN_Y` and
  header corrected; the golden trace RE-FROZEN via the independent
  generator with a dated provenance note in `meta.provenance_2026_10_21_r6`
  (source-fidelity correction, explicitly not tuning); the scenario-author
  policy parameter JUMP_FACTOR re-synchronized to the corrected geometry
  (0.25 → 0.30 — it was synchronized to the mistaken y=1 height; any
  factor producing ≥3 squashes + the death + the full spawn stream is a
  valid frozen scenario; the factor is now recorded in meta); new trace:
  3600 ticks, 45 spawns, 91 events (3 squashes at 995/1938/1990, death at
  2703). `3d-feasibility.md` item 1 corrected with the full derivation;
  the stale "23 spawns / seed 0x8" RNG note reconciled.
- **Content frames**: `squash_moment` and `retry_overlay` re-frozen from
  the corrected windowed run (mobs at y=0); `final_frame` verified
  BIT-IDENTICAL (end-state scene has no live mobs in frame) and kept.
  SHA256SUMS updated; manifest provenance records the dated correction.
- **Related coordinate paths rechecked** (R6 named these; stated as
  verified vs approximation):
  - SPAWN RECTANGLE x/z: unaffected — SpawnPath x/z = 0 (Main.tscn line
    128 transform), so only the y was wrong. Verified.
  - Player node-origin vs sphere-center: AGP integrates the SPHERE CENTER
    (rest y = radius). Source: the sphere shape (r=0.792278, local offset
    0.771765) rests with its center at radius height — node origin at
    center − 0.771765 (≈2 cm). AGP's model matrix places the model at
    center − radius; the 2 cm model-origin nuance REMAINS the documented
    approximation (collision semantics, which is what the detectors and
    squash classification consume, are center-based and exact).
  - MobDetector cylinders / mob box: player-local and velocity-yaw frames
    as implemented, validated by the F03/R4 probes; unchanged.
- **Verification**: first spawn y=0 in the regenerated trace; the Rust sim
  replays the regenerated trace BIT-EXACT (`golden_trace_replay_matches_
  reference` green, 0 position mismatches); all stomp/side/death/retry and
  R4-01 death-sequence controls pass; headless frames bit-exact vs the
  re-frozen goldens; headless case exit 0 (windowed in the final matrix).

### F829-R6-02 (P2) — composed-pixel gate missed 3D bleed and displacement

- **Root cause**: the analyzer SEARCHED the observed image for the first
  dark 2D row and accepted that as the expected top (a displaced composite
  redefined the expectation), and the clip check only looked for the 2D
  clear color outside the rects — 3D sky-colored pixels outside the 3D
  rect were invisible. R6's bound corrupt inputs passed: (1) padding
  strips recolored to sky (25,600 px outside) → clip_ok stayed true;
  (2) both views shifted down 80 px → views_top=178 accepted.
- **Fix**: expected geometry is now INDEPENDENTLY ANCHORED from the
  spike's own layout and the screenshot scale factor — padding 10,
  control band 31 (the iced default-theme button row), spacing 8, Fixed
  (320) view heights → expected_top = 49 × scale (98 physical @2x,
  matching the real capture exactly). The observed top is diagnostic only;
  a new `position_ok` fact requires |observed − expected| ≤ 2·scale+2 px
  and the expected rects to fit the image. View-presence checks sample the
  ANCHORED rects (a displaced view no longer re-anchors them). The clip
  check now observes BOTH views' signatures — the 2D clear color (20,23,31)
  ±16 AND the 3D sky color (165,167,171) ±20 — in the gap band, padding
  columns, and control band (<1% each).
- **Permanent controls** (`tests/composite_gate_contract.rs`, driving the
  SAME consumed analyzer with constructed composites): normal positive
  (green), transparent negative (red, existing case sabotage retained),
  3D-bleed negative (R6's exact transform: sky-recolored padding strips →
  clip false), displaced-views negative (R6's exact transform: both views
  shifted down 80 px → position_ok false, gate red), resize/DPI control
  (1720×1400 @2x → green).
- **Live evidence on the new code (bounded, environment-limited)**:
  three bounded windowed attempts on the r5 code. Attempt 1 stalled inside
  the initial screenshot readback (>9 min under host throttling; the
  documented E14/R5-01 environment behavior) and was killed at its 600 s
  bound. Attempt 2 captured the REAL initial composite
  (`scratch/829/r5/iced-positive-b/initial_composite.png`) and the anchored
  analyzer accepts it EXACTLY (expected_top=98 = observed_top=98,
  f_v2=0.973, f_sky=0.182, v3_colors=27, clip ~0 — verified by running the
  real capture bytes through the consumed analyzer), then stalled in the OS
  CLICK-delivery phase (the fallback clicker's FindWindow/powershell path
  is session-flaky under load; the case itself has no internal deadline —
  the R5-01 unbounded-wait debt) and was killed at its bound. Attempt 3 did
  not run; per the bounded-wait rule this is documented rather than
  retried indefinitely. What is verified on the new code: all five
  permanent gate controls (unit, constructed composites through the same
  analyzer), the real captured composite through the consumed analyzer,
  and the case-level positive/negative domain from r4 (same sabotage path).
  What remains environment-blocked: a single full case-level windowed
  positive completing all three moments on the r5 commit (input delivery,
  not rendering — the case produced its window, keyframes and initial
  composite normally).

### Diagnosis extension (multiple failed final reviews)

- **Actual source local/global coordinates**: the y=1 misread shows a
  class of error invisible to implementation-level consistency checks —
  every layer agreed (sim, generator, trace, docs) because they shared the
  ONE wrong reading of the source. Binding: source-constant claims must
  carry the frozen call path they were derived from (now in the SPAWN_Y
  comment, the generator header, and 3d-feasibility), and engine-executed
  source fixtures outrank textual reading (R6's Godot receipt).
- **Both-views clip + anchored geometry**: a gate that SEARCHES the image
  for its expectation accepts whatever it finds; expectations must be
  anchored in the producer's own layout contract. Binding: the composite
  analyzer anchors geometry from the view() layout constants, observes
  out-of-bounds content for BOTH views, and the position/clip facts are
  permanent tested controls.
- **Committed-delivery vs ignored working files**: `git status` clean is
  not the release boundary — ignored runtime inputs are invisible to it.
  Binding: the delivery-closure test enumerates the required-input
  inventory and asserts tracked-and-present per file; review evidence must
  be rebound to committed-only exports (the export replay is the closing
  check, recorded in result.md).
