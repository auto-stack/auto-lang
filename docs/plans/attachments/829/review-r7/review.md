# PLAN-829 — targeted confirmation re-review R7

2026-10-21 (Asia/Shanghai). **Outcome: PASS** (all F829-R6-01..03 closed;
iced case-level live rerun remains a merge-owned item with narrowed scope —
input delivery only, not rendering/gate logic). Scope: closure of R6
findings + adjacent no-regression. Reviewer: independent R7 agent; no
worktree file modified (clean before and after); evidence only under this
directory; AGP-scoped commands only.

## Baseline

- Worktree `D:/autostack/.wt/lang-829/auto-lang`, branch `plan-829-dev`.
- HEAD `814f9bf01aa14eadcd9e851bc4cce7ac8caa360b` (claimed 814f9bf01; matches).
- R5 baseline `b84c6d91e`; plan base `fdb1fad2ce`. Tree clean at entry and
  exit; diff vs base AGP-only (0 files outside AGP).
- r5-repair commit touches 15 files: sim.rs (SPAWN_Y 1→0 + derivation),
  iced_spike.rs (anchored clip gate), testdata (golden_frames/.gitignore
  negation, manifest.json now tracked, SHA256SUMS, re-frozen
  squash_moment/retry_overlay, regenerated trace + generator provenance),
  new tests/delivery_closure.rs + tests/composite_gate_contract.rs,
  3d-feasibility.md, evidence-notes E17, result.md. Lock unchanged
  (`6cd345763c2d…`).

## Per-finding closure verdicts

### F829-R6-03 (P1) committed delivery closure — CLOSED

- Manifest now TRACKED: `git ls-files` lists it; `git check-ignore` exit 1
  (not ignored); `git cat-file -e HEAD:<path>` succeeds. Nested
  `testdata/golden_frames/.gitignore` (`!manifest.json`) documented.
- Delivery-closure test binds a 24-path required inventory (golden
  manifest + SHA256SUMS + all manifest-declared frames, both frozen traces,
  both fonts, both audio assets, both GLBs, 3 integrity receipts) with
  exist + tracked + not-ignored assertions, plus a manifest↔files
  cross-check. DISCRIMINATES: in a throwaway git repo with the manifest
  left untracked, the inventory test goes RED (frames cross-check stays
  green).
- R6's exact closing check executed by this review: `git archive HEAD`
  export → standalone build (`--locked --offline`) → ag2d-breakout AND
  ag3d-squash headless from committed-only content → BOTH exit 0, trace
  bit-exact (max_pos_diff 0.0, event_sequence_exact), export SHA256SUMS
  6/6 OK. `compare=Some` holds by the verdict predicate (headless pass
  requires non-empty all-ok golden checks; `None` would record
  (-1,-1,false) and fail). Note: export fingerprint embeds the parent
  checkout's HEAD (`-dirty`) because build.rs resolves the enclosing git
  repo — an artifact of the export location, not a delivery defect.
- Independent ignored-file audit: only AGP-local `scratch/` (all runtime
  references are write paths) and two godot-engine-reference research
  artifacts (read by no gate) are ignored outside target/; no other
  required-but-ignored input exists.

### F829-R6-01 (P2) source spawn height — CLOSED

- Independent source derivation verified from frozen testdata:
  `Main.gd:36-37` passes `mob_spawn_location.position` (LOCAL, not
  global_position) to `mob.initialize`; `Main.tscn:131` parents
  SpawnLocation under SpawnPath (transform y=1, line 128); the mob is
  `add_child`ed under Main — so spawn y=0. Matches R6's Godot fixture
  receipt (`actual_spawned_mob_y=0`, four seeds).
- Code: `SPAWN_Y = 0.0` with the full call-path derivation in the const
  comment; first spawn in the new trace is (14, 0, -0.35427094) vs R6's
  measured wrong value (14, 1, -0.35427094) — same RNG stream, only y
  corrected.
- Regeneration provenance: generator header carries a dated CORRECTION
  block (source-fidelity correction, explicitly not tuning); trace
  `meta.provenance_2026_10_21_r6` records the derivation + fixture
  receipt; scenario-author policy parameter JUMP_FACTOR 0.25→0.30
  documented (re-synchronized to corrected geometry; any factor producing
  the documented scenario shape is valid; recorded in meta). NOT a
  tuned-to-pass artifact: the height comes from the source call path, not
  from fitting.
- Canonical replay bit-exact vs the new trace (suite + both probe modes,
  max_pos_diff 0.0); stomp/side/death/retry controls all green; trace
  shape 45 spawns/3 squashes/1 death/42 despawns = 91 events, reconciled
  in the test header and result.md (dated).
- Docs: 3d-feasibility.md item 1 CORRECTED with the full derivation; sweep
  finds no remaining y=1-as-spawn-height claim outside historical/correction
  context. Related coordinate paths documented as verified-vs-approximation
  in E17 (spawn rectangle x/z unaffected — SpawnPath x/z = 0; player
  node-origin vs sphere-center ~2 cm nuance REMAINS the documented
  approximation).
- DISCRIMINATES: reverting SPAWN_Y to 1.0 in a throwaway copy turns
  `golden_trace_replay_matches_reference` and
  `spawn_stream_reproducible_across_runtimes` RED.
- Content frames: squash_moment + retry_overlay re-frozen (mobs at y=0 —
  rendering changed), final_frame bit-identical and kept; SHA256SUMS +
  manifest provenance dated.

### F829-R6-02 (P2) anchored clip gate — CLOSED (case-level live rerun remains merge-owned)

- Fix verified at source: expected geometry INDEPENDENTLY anchored from
  the spike's own layout (padding 10, control band 31, spacing 8,
  Fixed(320) views) x scale — `expected_top = 98 physical @2x`; the image
  search is now diagnostic-only; new `position_ok` fact
  (|observed-expected| <= 2*scale+2, rects fit); clip observes BOTH
  signatures (2D clear (20,23,31)+/-16 AND 3D sky (165,167,171)+/-20) in
  gap/padding/control bands. The analyzer is a public function taking raw
  bytes — the contract test drives the SAME consumed code path.
- Permanent controls (`tests/composite_gate_contract.rs`, 5 tests, all
  green): normal positive (incl. position_ok), transparent negative,
  R6's EXACT 3D-bleed corrupt input (sky-recolored padding strips →
  clip false), R6's EXACT displaced-views input (both views -80 px →
  position_ok false), resize/DPI control (1720x1400@2x green).
  DISCRIMINATES (this review's mutations in a throwaway copy): clip
  weakened to 2D-only → bleed test RED; position gate forced true →
  displaced test RED.
- Real-capture analyzer run INDEPENDENTLY reproduced by this review: the
  work's real `initial_composite.png` bytes
  (`scratch/829/r5/iced-positive-b/`) through the committed analyzer →
  `expected_top=98 observed_top=98`, f_v2=0.973, f_sky=0.182,
  v3_colors=27, clip~0, ok=true.
- Case-level live evidence on the new code (work, bounded): attempt 1
  stalled inside the initial screenshot readback >9 min under host
  throttling, killed at its 600 s bound; attempt 2 produced the window,
  GPU keyframes AND the real initial composite (above), then stalled in
  OS click delivery (the R5-01 unbounded-wait debt — no internal
  deadline), killed at its bound; attempt 3 not run per the bounded-attempt
  rule. The committed E17 states this honestly (the earlier draft's
  stronger claim was corrected before commit).
- No new unbounded wait introduced by this repair: the phase flow is
  unchanged since r4; the screenshot readback is synchronous-in-loop
  (completes or is killed externally); the only unbounded wait remains the
  pre-existing R5-01 click-phase debt.

## Diagnosis extension (E17) — VERIFIED

E17 names all three missed invariants with causes and binds permanent
controls: (1) committed-delivery vs ignored working files (git-status
clean != delivery boundary; binding: delivery_closure inventory + audit
rule), (2) source local/global coordinates (every layer agreed because
they shared ONE wrong source reading; binding: source-constant claims need
independent source re-derivation, plus the verified-vs-approximation
coordinate recheck), (3) both-views clipping / independently anchored
geometry (image-search redefined the expectation; binding: expected
geometry from layout x scale, both signatures observed, bound corrupt
inputs as permanent controls). The standing rule (gates name invariants,
replays exercise full invariants) is restated. All named controls exist
and discriminate (mutation evidence above).

## Iced live-evidence disposition

The gate's DISCRIMINATING POWER on the new code is fully established:
5 unit controls on the exact consumed analyzer + real-capture acceptance
(independently reproduced) + 3 weakening mutations red + the transparent
negative's facts verified discriminating. What is NOT demonstrated on the
new code is a single full case-level windowed positive completing all
three moments end-to-end — blocked by agent-session input delivery /
readback throttling, NOT by rendering (window, keyframes, initial
composite all produced normally). SUFFICIENT for R7 closure with an
explicit remaining merge-owned item: run iced-shared-device windowed
(positive + AGP_SABOTAGE_ICED=transparent_blit negative) on the frozen
commit in an interactive session, after zombie cleanup (taskkill
graphics-probe.exe). This narrows R5's caveat from "gate evidence" to
"end-to-end sequence completion".

## No-regression spot checks

| Check | Result |
|---|---|
| `cargo test --locked` | exit 0, **104/104** (lib 28, breakout 11, cache 3, iced_seam 5, delivery_closure 2, composite_gate 5, prototype 16, squash 17, threed 10, verifier 7) — matches the claimed 104; zero warnings |
| `cargo fmt -- --check` | exit 0 |
| probes in worktree: breakout + squash, window AND headless | exit 0 x4 |
| probes: verifier-negative-controls / metrics / lifecycle / shared-runtime | exit 0 x4 |
| probe `independent-consumer` from clean state (consumer-a-target + relocate deleted first) | exit 0 (65 s), ok=true, zero mismatches, fingerprint `814f9bf01aa1` both sides |
| committed-only export replay (R6 closing check) | both games exit 0, bit-exact, golden integrity OK |
| `research/result.md` completeness | complete and consistent (head + all 6 sections + tail; final counts 104; dated historical lines kept; rebuild after the truncation incident produced an intact file, 250 lines) |
| tree / diff | clean at 814f9bf01; AGP-only |

## New findings / observations

- **F829-R7-01 (P3, trivial)**: `result.md` E15 narrative still quotes
  "45 events (22/3/1/19)" as the R2/R3-era correction — history as written,
  but two re-freezes old; the dated r4 and r6 sections directly above/below
  carry the current 91-event facts. Optionally add "(superseded by r4/r6
  re-freezes)" at deposition. Not blocking.
- **F829-R7-02 (P3, carried)**: R5-01 (unbounded click-phase wait) remains
  open and was directly hit by the work's bounded attempt 2; keep in
  KNOWN-DEBT-AND-RISKS.md. F829-R5-02 (zombie runbook) applies to the
  merge-owned iced rerun.
- No other new findings. The r5-repair commit is surgical (641 insertions /
  15 files), no dependency/lock/root changes, no fmt/warning drift.

## Recommended plan status / next

- All F829-R6-01..03 **CLOSED**. Recommend `status: execution_done`, T-10
  (final review) **PASS** at frozen commit
  `814f9bf01aa14eadcd9e851bc4cce7ac8caa360b`.
- T-11 merge-owned gates (pending, on the frozen commit): full AGP suite +
  10 probes; iced-shared-device windowed positive + transparent_blit
  negative in an interactive session (zombie cleanup first) — the one
  remaining live evidence item; real consumer relocation; migration
  handoff authorization; SD-01/02/03 deposition (update SD-03 IH-01 with
  the composed-pixel gate per F829-R5-03, plus the R5-02 runbook note and
  R7-01 annotation); register debts (R5-01, Godot character-level engine
  comparison); archive; wt-guard clean + worktree/branch cleanup.
- Do NOT reopen any F829-R1/R2/R4/R6 finding.
