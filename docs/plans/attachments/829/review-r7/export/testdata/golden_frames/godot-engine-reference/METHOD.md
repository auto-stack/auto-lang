# Godot original-engine ground-truth capture (PLAN-829 F07, 2026-10-21)

## What this is

The frozen `3d/squash_the_creeps/` project (godot-demo-projects@b761b4cd) run
in the ORIGINAL ENGINE — **Godot 4.4.1-stable win64** — with the frozen
canonical action stream (`golden_squash_trace.json` actions) injected per
physics frame, capturing viewport frames + a scene-state trace.

## Method

- Binary: `Godot_v4.4.1-stable_win64.exe`, downloaded from the official
  GitHub release (api.github.com asset endpoint; 69,065,928 bytes;
  github.com release pages were unreachable from this network, the API
  asset endpoint worked). Kept at `scratch/829/godot-gt/` (gitignored).
- Driver: `capture.gd` (SceneTree main-loop script, preserved in
  `scratch/829/godot-gt/project/capture.gd`): loads the frozen actions,
  presses/releases the frozen input actions per physics frame
  (`Input.action_press/release`, real `is_action_just_pressed` semantics),
  records player position / score label / alive flags / mob count per
  physics tick, and saves viewport PNGs at ticks 60/271/1372/1402/3599.
- Run: windowed, 900x600 viewport, Forward+ renderer, full 3600 physics
  ticks (~30 s). Log: `godot_run.log`.

## What was verified against the AGP oracle

- **Engine runs the frozen project end-to-end** with our exact action
  stream; scene state facts captured from the live scene tree.
- **Rest-height convention**: Godot `Player.position` (CharacterBody3D
  ORIGIN) rests at y≈0.02; AGP's `player` is the collision SPHERE CENTER
  (y≈0.79). Horizontal aim/trajectory comparisons are the meaningful ones;
  the vertical offset is a frame convention, documented.
- **Jump apex**: Godot body-origin apex 1.51 m (t=175) ⇔ sphere-center apex
  ≈2.28 m vs AGP 2.39 m — ≈0.1 m agreement (integrator/timing differences
  within the documented single-pass approximation).
- **Death/retry semantics confirmed by the original engine**: player freed
  at t≈865, `Retry` overlay with dimmed scene + centered "Press Space or
  Enter to retry" + Score label — the captured `death.png` matches AGP's
  retry-overlay frame semantically (dim, text, HUD placement).
- **Gameplay-outcome divergence (documented, expected)**: under real Jolt
  physics the frozen oracle-player policy scores **0** squashes and dies at
  t=865, vs AGP's approximation-scenario 3 squashes / death at 1372. The
  policy was tuned against the documented approximation; the divergence is
  the approximation boundary made visible, not a silent error.

## Limitations (honest)

- **Character models do not appear in the captured frames.** The `.godot`
  resource-import cache could not be built in this environment: three
  headless import methods (`--import`, `--editor --quit-after` short/long)
  all failed to produce the `.godot/imported/*.scn` files — the MusicPlayer
  autoload fails to load (its ogg is also un-imported) and the editor
  import scan aborts (see `godot_run.log` / import logs). The Player/Mob
  scenes therefore instantiate without their GLBs. Environment, lighting,
  shadows, sky, HUD, and the retry overlay DO render and are compared.
- Ground-truth frames are therefore evidence for **static scene + UI +
  lifecycle semantics**, not character animation. Character-level
  original-engine comparison remains open (needs a machine where the
  editor import scan completes; e.g. open the project in the editor once).

## Decision item for review R2

The PASS-gate content oracle (`testdata/golden_frames/ag3d-squash`,
`ag2d-breakout`) is implementation-derived (bit-exact canonical captures,
frozen with sha256 + tolerances). This original-engine capture is the
independent baseline evidence that exists today; per the Plan's oracle
rules, R2 may either accept the documented alternative oracle (independent
Python reference + frozen source constants + frozen content frames +
engine-level ground truth for trajectory/semantics) or require completing
the character-level engine comparison as a follow-up item.
