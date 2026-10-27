# PLAN-829 — targeted confirmation re-review R5

2026-10-21 (Asia/Shanghai). **Outcome: PASS** (all F829-R4-01..05 closed;
iced live rerun environment-blocked but closure verified via same-machine
recorded evidence + code verification + compiled-in permanent control; one
P3 observation carried forward). Scope: closure of R4 findings only +
adjacent-gate no-regression. Reviewer: independent R5 agent; no worktree
file modified (clean before and after); evidence only under this directory;
AGP-scoped commands only, no root tiers.

## Baseline

- Worktree `D:/autostack/.wt/lang-829/auto-lang`, branch `plan-829-dev`.
- HEAD `b84c6d91e83a00d4101e6e42403820127516c061` (claimed b84c6d91e; matches).
- R3 baseline `077de3b6697c69053c24f4067bf73188ca90ab62`; plan base
  `fdb1fad2ce7afa124adfa3080323496df384e670`.
- Tree clean at entry and exit; diff vs base AGP-only (0 files outside
  `experimental/autographics-prototype/`).
- r4 commit touches 11 files (sim/live/scene/iced_spike/iced_shared_device/
  squash_game/consumer-adjacent sources, squash_contract tests, evidence
  notes E16, result.md, one golden frame + SHA256SUMS). AGP lock sha256
  unchanged: `6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`.
- Review inputs: `attachments/829/review-r4/review.md` (read in full).

## Per-finding closure verdicts

### F829-R4-01 (P2) player role identity — CLOSED

- Fix verified at source: `Scene::first_spawned` records the session's
  first-spawned id (set once, survives destruction/reuse);
  `project_instances` classifies by `Some(o.id) == player_id` — no mutable
  alive-set head. Independent-consumer rule stated (first Spawn = player).
- Bound probe replayed: canonical trace to death (tick 1372), player id
  (index0, generation0) destroyed; permanent test
  `r4_player_role_identity_stable_through_death_and_reuse` covers
  before-death, after-death all-Mob (4+ live mobs), leading-mob deletion
  (per-id kinds stable while population shrinks), empty scene (nothing
  projects, anchor inert), restart (new session, new anchor), and
  independent-log-replay classification (per-id kinds reproduced;
  index+generation identity comparison). All green in the suite.
- Discrimination: reverting `project_instances` to the first-ALIVE rule in
  a throwaway copy turns the test RED.
- Golden frame: `retry_overlay.png` is the ONLY re-frozen frame
  (SHA256SUMS diff = exactly 1 line; squash_moment/final_frame and all
  breakout frames unchanged). Independent pixel diff old-vs-new (old frame
  extracted from git `077de3b66`): 3563 changed px = 0.66% of the frame,
  one compact bbox (195,408)-(295,474) = 101x67 px, old side
  orange-dominant (1019 px), new side dark-blue-dominant (1521 px) —
  exactly the E16 claim (misclassified mob region, orange player pixels
  replaced by mob pixels). Source contract (`Player.gd:68-70` die() =
  queue_free() ⇒ no player image after death) independently establishes
  the expectation; tolerances/sha256 discipline/corruption control
  unchanged; re-freeze dated in manifest.json provenance. RULING: honest
  source-derived expectation repair, NOT an oracle redefinition.

### F829-R4-02 (P2) Space retry — CLOSED

- Fix: `key_event` takes the dead-state; dead Space = retry effect with NO
  jump edge armed (space_held updated before return — held-through-reload
  cannot fire a stale jump); dead Enter = retry; living Enter = nothing
  (machine-level, not host-gated); host passes `driver.dead` through the
  real winit keyboard path (squash_game.rs:1257-1261).
- Permanent controls: 3 new lib tests + `r4_dead_space_and_enter_retry_through_host_mapping`
  (dead Space/Enter retry, visibility gating alive/not-visible, living
  Space jump-only, held/release across death+reload, first input after
  reload). All green.
- Discrimination: reverting the Space dead-branch to the r3 state turns
  both the contract test and the lib tests RED.

### F829-R4-03 (P2) detector coordinate frames — CLOSED

- Fix: `rotation_y(player_facing_yaw - mob_yaw)` on the player-local
  offset (relative yaw), with the derivation in a comment.
- Permanent control: `r4_detector_classification_rigid_rotation_invariant`
  — the R4 bound probe (90° global rotation invariance, body-free) plus
  4 encounter classes (bound-probe miss / frozen F03 hit / corner hit /
  y-channel miss) x 5 global rotations (0, pi/2, pi, -1.1, 2.5), plus
  absolute-orientation-independence assertions. Coordinates reuse frozen
  probe geometry (no tuning). All green.
- No golden regeneration: canonical replay still bit-exact
  (`max_pos_diff 0.0`, `event_sequence_exact`, death at tick 1372 —
  identical to R2's recorded run; the frozen scenario dies at
  facing-zero geometry, so the frame fix is a no-op on the canonical
  path — verified by the suite and both squash probe modes).
- Discrimination: reverting to `rotation_y(-mob_yaw)` turns the test RED.

### F829-R4-04 (P2) snapshot rotation — CLOSED

- Fix: snapshot now serializes the sign-canonicalized quaternion
  (q ~ -q canonicalized by first non-zero component; -0.0 normalized), so
  rotation-only state is distinguishable and matrix-equivalent orientations
  stay equal. No frozen hash constants needed re-basing (all consumers are
  equality-based).
- Permanent control: `r4_snapshot_distinguishes_rotation_only_changes`
  (bound probe rotationY=180°, unchanged equality, sign-flipped equivalent
  equality, identity-restore). Green; shared-runtime probe exit 0 after
  the change.
- Discrimination: removing the `q(...)` field from the snapshot format in
  a throwaway copy turns the test RED.

### F829-R4-05 (P1) composed-content gate — CLOSED (live rerun environment-blocked; see note)

- Fix verified at source: per-moment `window::screenshot` (iced 0.14
  in-compositor readback — re-runs the FULL UI draw through the same
  engine/compositor the window presents; NOT a re-render of our own view
  textures), analyzed by `analyze_composite_shot`: (1) non-blank >2% vs
  calibrated window bg, (2) controls band (solid fill + dominant color),
  (3) 2D view at layout-derived rect with source-anchored clear color
  (20,23,31)+/-16, (4) 3D view (source-anchored sky (165,167,171)+/-20 AND
  >=4 quantized colors), (5) clip (view-2d background <1% in gap/ctrl/pad
  bands). Missing moment = fail. Wired into the verdict
  (`composite_content_ok && pass`) — real assertions, not decorative.
- R4 transparent-final-fragment negative reproduced by the work, twice,
  on this machine (`scratch/829/r4/iced-transparent-neg2/`): exit fail,
  content ok=false at ALL three moments (f_v2=0.000, f_sky=0.000,
  v3_colors=0) WHILE the F05 draw counters stay green (2->104->111
  advancing) — exactly the R4 counterexample. Positive
  (`iced-positive5/`): all three moments content ok=true, counters
  advancing. Artifact fingerprints read `077de3b6697c-dirty` = the r4
  repair code 5 minutes before it was committed unchanged as b84c6d91e
  (worktree clean at review entry; committed lock matches).
- Permanent negative control compiled into shipped code:
  `AGP_SABOTAGE_ICED=transparent_blit` forces the exact R4 fragment
  mutation (env-sabotaged shader swap at pipeline creation).
- Click/exit interaction: click dispatch was re-phased to AFTER the
  initial screenshot completes (readback blocks the loop; documented in
  code and E16). Exit boundedness: the work's recorded runs complete
  normally through pre_close + finalize + window close with reports
  written; the sequence is phase-driven with no unbounded new wait.
- **R5 live rerun note**: this review session could not complete even the
  FIRST screenshot readback in 270 s of observation (initial keyframes
  render; `initial_composite.png` never appears). The work measured the
  same readback at 40-60 s under throttling in an interactive session.
  Root cause class: agent-session compositor visibility (same family as
  the R3 click flake) — NOT a product defect: the case's own GPU keyframes
  (positive + transparent_blit negative) as a merge-owned gate in an
  interactive session, after checking for stale same-title windows.

## No-regression spot checks

| Check | Result |
|---|---|
| `cargo test --locked` | exit 0, **97/97** (lib 28, breakout 11, cache 3, iced_seam 5, prototype_contract 16, squash 17, threed 10, verifier 7); zero compiler warnings — matches the claimed 97 |
| `cargo fmt -- --check` | exit 0 |
| probe `ag2d-breakout` window + headless | exit 0 both (`breakout-window/`, `breakout-headless/`) |
| probe `ag3d-squash` window + headless | exit 0 both (`squash-window/`, `squash-headless/`); trace bit-exact (max_pos_diff 0.0, death tick 1372), re-frozen retry_overlay golden accepted by the content gate |
| probe `verifier-negative-controls` | exit 0, 5/5 gates |
| probe `metrics` / `lifecycle` / `shared-runtime` | exit 0 / 0 / 0 |
| probe `independent-consumer` from clean state (consumer-a-target + relocate deleted first) | exit 0, `ok=true`, zero mismatches, fingerprint `b84c6d91e83a` both sides |
| probe `iced-shared-device` windowed (positive + transparent negative) | live rerun environment-blocked (compositor screenshot readback does not resolve in this agent session); closure per the R4-05 verdict above |

## New findings / observations

- **F829-R5-01 (P3, carried forward, still open)**: iced phase 2 still waits
  unbounded on `click_observed` (R3-01 debt unchanged by r4 — r4 re-phased
  click dispatch after the screenshot, narrowing the race, but did not add
  the fail-fast timeout). Keep registered in KNOWN-DEBT-AND-RISKS.md.
- **F829-R5-02 (P3, process-hygiene note, not a code defect)**: killing the
  iced case from outside (tool timeouts) leaves zombie `graphics-probe.exe`
  windows with the case's fixed title; a later run's `FindWindow`-based
  clicker then targets the dead queue and can never observe a click.
  Operators (incl. merge gate) must `taskkill /F /IM graphics-probe.exe`
  and confirm zero instances before a windowed iced run. Worth one line in
  the README/handoff runbook; no code change required for closure.
- No other new findings. The r4 commit is surgical (1022 insertions across
  11 files), introduces no dependency/lock/root changes, no fmt/warning
  drift, and every new assertion was shown to discriminate.

## Recommended plan status / next

- All F829-R4-01..05 **CLOSED** (R4-05 with the recorded live-rerun caveat,
  merge-owned re-queue). Recommend keeping `status: execution_done` with
  T-10 (final review) **PASS** at frozen commit
  `b84c6d91e83a00d4101e6e42403820127516c061`.
- T-11 merge-owned gates (all pending, run on the frozen commit): full AGP
  suite + all 10 probes; **iced-shared-device windowed positive AND
  `AGP_SABOTAGE_ICED=transparent_blit` negative in an interactive session,
  after zombie cleanup** (this is now the one live evidence gap); real
  consumer relocation; migration/write-surface handoff authorization;
  deposition of SD-01/02/03 (r4 wording reconciliation verified in R4's
  scope — SD statements reviewed at R4, unchanged by r4 except
  fact-alignment); register F829-R5-01 (and re-confirm R3-01 dedup) +
  F829-R5-02 runbook note + the character-level Godot engine comparison
  (R2 decision item a) in KNOWN-DEBT-AND-RISKS.md; archive; wt-guard clean
  + worktree/branch cleanup.
- Do NOT reopen any F829-R1/R2/R4 finding; R3-01/F829-R5-01 remains the
  single registered debt item in this area.

- **F829-R5-03 (P3, deposition item)**: the r4 commit did not update the
  SD drafts, and SD-03's IH-01 paragraph still cites only the in-band
  composite counters as the blank-composite evidence — true but now
  under-recording: the composed-pixel content gate (iced `window::screenshot`
  readback; non-blank / controls / both views at layout rects / clip, at all
  three moments, plus the permanent `transparent_blit` negative) is the
  stronger verified fact R4 demanded. R4 explicitly conditioned SD
  reconciliation on these findings. Not a code/acceptance gap — the SD
  drafts are not yet deposited — but the deposition step (T-11) MUST update
  SD-03 IH-01 (and add the E16 detection rule reference) before writing
  `docs/specs/auto-graphics/`. Verified SD-01/SD-02 contain no
  R4-contradicted statements (sweep: no role/retry/detector/snapshot claims
  beyond what r4 fixed; SD-02's lifecycle/upload facts re-verified green this
  round).
