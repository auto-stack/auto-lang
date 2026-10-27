# PLAN-829 — independent final review R2

Date: 2026-10-21 (Asia/Shanghai). **Outcome: needs_fix** (narrow scope: one
P2 gate-wiring defect introduced by the r1 repair, plus P3 fact/robustness
corrections). Scope: final, second independent review (R1 = needs_fix;
work r2 repair batches 03f4999f4 / b5c1ed1af / 1fc3f6116 / 95ccfcc5c).
Reviewer: independent R2 agent with zero prior context; no second agent or
session was created. No implementation file was modified: all counterexample
replays ran against regenerated injection copies under this attachment dir
(since removed; mutation diffs recorded below) or via built-in sabotage env
controls. No root cargo tiers were run; only AGP-scoped commands.

## Frozen baseline

- Plan revision: 1; entry status executing (R1 needs_fix, work r2 pass
  handoff). Input Plan SHA256
  `3f53bb418406f20d3cbb83e8f5b03189bb41021c809c712ce8f0125a33026681`
  ([input-sha256.txt](input-sha256.txt)).
- Worktree: `D:/autostack/.wt/lang-829/auto-lang`; branch `plan-829-dev`.
- Reviewed commit: `95ccfcc5c18a6c4a3051f80ee38ab19bf05d67f7` (= claimed
  code_commit; `git rev-parse HEAD` verified). Base
  `fdb1fad2ce7afa124adfa3080323496df384e670` verified as merge base.
- Tree clean at entry AND after all review runs (`git status --short`
  empty; probe runs wrote only gitignored scratch/target + this attachment
  dir). Implementation worktree HEAD did not drift during review.
- Diff vs base: 123 files, +26260 lines, ALL under
  `experimental/autographics-prototype/` (AGP); root workspace/lock and
  production crates untouched (empty file list outside AGP).
- Reparse-point scan: no symlinks under AGP (`find -type l` empty);
  junction listing (`cmd dir /a:L`) over AGP root and each top-level
  subdirectory (src, assets, research, scratch, testdata, target) empty.
- AGP `Cargo.lock` SHA256 `6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`
  (deliberately changed from R1's `d9e1fdae…` by the F09 de-vendoring lock
  regeneration; consistent with the plan record).
- Dependency closure: `cargo metadata --locked --offline` — 478 packages,
  **zero path-sourced packages**, no `[patch]` entries; `agp` dependencies
  all registry-sourced (wgpu 27.0.1, winit 0.30.13, iced/iced_wgpu 0.14.0,
  iced_widget 0.14.2 stock). `patches/` tree is gone from the worktree.

## Reproduced evidence (all commands AGP-scoped; M = experimental/autographics-prototype/Cargo.toml)

| Check | Result |
|---|---|
| `cargo test --manifest-path M --locked` | exit 0, **90/90** (lib 24, breakout 11, cache 3, iced_seam 5, prototype_contract 16, squash 14, threed 10, verifier 7); **zero compiler warnings** |
| `cargo fmt --manifest-path M -- --check` | exit 0 |
| `git diff --check base...HEAD` | only `assets/breakout/fonts/FiraMono-LICENSE:21` trailing whitespace — frozen OFL bytes, the explicitly preserved upstream exception R1 already accepted |
| asset/source/font hash verification | `testdata/SHA256SUMS` 22/22 OK (from `testdata/`); `assets/SHA256SUMS` all OK; `testdata/breakout/FONTS.sha256` 2/2 OK (paths relative to `assets/`); `testdata/golden_frames/SHA256SUMS` 6/6 OK (CRLF-stripped) |
| probe `gpu-foundation` (headless) | exit 0 |
| probe `shared-runtime` (headless) | exit 0; snapshots_agree, events_equal, 3 stale-id rejections, late-event rejected, gpu_timing honestly false |
| probe `ag2d-breakout` (**headless**) | **exit 1** — every gameplay fact green (trace bit-exact max diff 0.0/10800, 210/210 sounds, 56/56 bricks, 3 resets reproducible) but the F07 golden gate can never be satisfied offscreen → F829-R2-01 |
| probe `ag2d-breakout` (windowed) | exit 0 (mode window 1800x1200 scale 2) |
| probe `ag3d-risk-slice` (headless) | exit 0 |
| probe `ag3d-squash` (headless) | exit 0; bit-exact (max_pos_diff 0.0), event_sequence_exact (full payload), spawn_stream_matches_golden, retry snapshots equal, music Playing |
| probe `lifecycle` (headless) | exit 0; threads sampled before/after workload/after teardown (4→21→13 this run), RSS trio, witness-derived view add/remove, cancellation callback acknowledged (`Err(BufferAsyncError)`), clean_exit after device drop |
| probe `metrics` (headless) | exit 0; instrumented rewrite bytes >0 both games, creations 0/frame, raw series persisted, fingerprints (agp_commit/lock/assets) attached |
| probe `verifier-negative-controls` (headless) | exit 0; all 5 gates positive+negative fired |
| probe `iced-shared-device` (**window**) | exit 0; real click via posted_queue through winit's real message path (2 presses), DPI 2.0, resize, 12 keyframes, composite counters [2d,3d] advancing at every moment (1→501→504), both negative gates fired |
| probe `independent-consumer` (headless, at entry) | **exit 1** — relocation build+run succeeded; single mismatch `fingerprints.agp_commit 1fc3f61164b0 vs 95ccfcc5c18a` because the A-side reuses the prebuilt `target/release` binary → F829-R2-02 |
| probe `independent-consumer` (after `cargo build --locked --release --bin graphics-probe`) | exit 0 |

Sabotage / discrimination replays (each: current code verified first, then the
r1-class mutation regenerated against CURRENT source in an injection copy —
no transplanted R1 copies; copy deleted after logging, mutations recorded here):

| Replay | Mutation (regenerated vs current source) | Result |
|---|---|---|
| F01 | `live.rs`: `let jump = self.jump_edge && !dead` → `let jump = !dead` (the r1 defect) | `f01_live_input_press_edge_semantics` **RED** (idle auto-jumps) |
| F02 | `core/time.rs`: `feed_capped` body → `return 1` (redraw-count clock) | `f02_live_tick_accumulator_cadence_invariant` **RED** + lib `capped_feed_is_cadence_invariant_and_bounded` **RED** |
| F03a | `sim.rs`: spawn aim `(self.player.x, self.player.z)` → `(0.0, 0.0)` | `f03_spawn_aim_tracks_the_actual_player_position` **RED** |
| F03b | `sim.rs`: driver detector channel disabled (`if squash.is_none() && !fatal` → `if false`) | `golden_trace_replay_matches_reference` + `death_stops_spawning_and_shows_retry_overlay` **RED** (2 tests); the isolated-geometry test stays green by design — the golden trace guards the wiring |
| F04 | `resources.rs`: key-owner binding check disabled | `f04_key_owner_bound_to_validated_owner` **RED**; leak/recreate tests stay green (they cover other invariants) |
| F05 | in-tree env control `AGP_SABOTAGE_ICED=blank_blit` (built-in; equivalent to R1's source mutation: skip the final composite draw, return true) | iced case exit **1**, `composite.ok=false`, counters `[0,0]` at all moments (report in `iced-sabotage/`) — R1's exact attack is now red |
| F06a | `witness.rs`: `Tracked::drop` decrement removed | `witness_detects_forget_leaks` **RED** |
| F06b | `lifecycle.rs`: `std::mem::forget(a); std::mem::forget(b);` at the end of the view cycle block (R1's retained-target injection, 20 targets total) | lifecycle case exit **1**, `witness_alive_end=21` vs baseline 1, `baseline_restored=false` (report in `lifecycle-sabotage/`) |
| F07 | injection copy: append 1 byte to frozen `ag3d-squash/squash_moment.png` | ag3d-squash case exit **1** (golden sha256 integrity → check (name,-1,-1,false) → verdict red) |
| F08 | in-tree env control `AGP_SABOTAGE_METRICS=1` | metrics case exit **1** (instrumented total vs raw-series cross-check fires) |
| F08b | `AGP_VERIFIER_BROKEN=<gate>` for all 5 gates | each exit **1**; unknown gate name exits 2 (also correct) |
| F11 | `sim.rs`: scene-delta submission disabled (`if !changes.is_empty()` → `if false`) | `f11_shared_contract_reconstructs_and_edits_3d_scene` **RED** |

Instrumentation audit: `AGP_TRACE` appears nowhere in committed code (one
historical mention in `research/evidence-notes.md` E14 stating the traces
were reverted). `println!`/`eprintln!` sites are live-mode usage hints and
error/diagnostic reporting only — no debug leftovers.

## F829-R1-01..11 closure verdicts

| Finding | R2 verdict | Basis |
|---|---|---|
| R1-01 P1 LIVE jump input | **CLOSED** | `LiveSquashInput` press-edge state machine (space_held/edge consumed per tick, dead gating) wired to real `PhysicalKey` events in the live host; regenerated r1-defect mutation → red; idle/held/re-press/dead cases all asserted |
| R1-02 P2 redraw-count timing | **CLOSED** | `TickClock::feed_capped` elapsed-time accumulator with cap+debt-drop; BOTH live hosts (breakout:1054/1163, squash:1143/1297) wire it; 30/60/120-cadence invariance + overload behavior asserted; mutation → red in two independent tests |
| R1-03 P2 Squash source-state paths | **CLOSED** | Spawn aim targets actual player position (sim.rs:384); five-cylinder `detector_fatal` channel wired in the driver tick (sim.rs:535-546); facing yaw / 4x-1x animation clock / jump pitch (Player.gd:66) / mob spawn-age phase all implemented and asserted against frozen constants; mutations → red (driver-level via golden trace) |
| R1-04 P2 cache ownership/identity leak | **CLOSED** | keys bound to validated owner token (resources.rs:106), same-key size change = miss-and-recreate, entry IDs freed on eviction/release; R1's exact A→B-valid-handle attack and 64-cycle leak loop are permanent GPU-gated tests; mutation → red |
| R1-05 P1 Iced blank-composite false positive | **CLOSED** | Real composite-path counters incremented only adjacent to the actual blit draw; per-moment monotonic gate in the verdict; R1's exact blank-blit mutation → exit 1; GDI capture hang honestly documented with in-band deterministic alternative; windowed positive still passes |
| R1-06 P2 lifecycle teardown oracle | **CLOSED** | DropWitness-derived drop counts (forget-leaks observable), three-phase sampling (before workload / after workload / after device destruction), real game resources in 20 restart rounds, awaited map_async cancellation acknowledgement, clean_exit after teardown inside all_ok, no process::exit on the iced path; R1's exact mem::forget injection → exit 1 |
| R1-07 P2 source oracle / pixel validation | **CLOSED with documentation nits** | Manifest reconciled in place (dated correction: seed 0x1B, 60-tick cadence, matches frozen trace — verified trace spawns at ticks 60,120,…); runner compares FULL event payloads (kind+tick+index+score) and FULL spawn tuples (pos+yaw+speed+vel+tick); content oracle = per-pixel mean/P99 against sha256-pinned frozen frames wired into BOTH game verdicts (corruption → red); original-engine ground truth captured (see decision item a). Nits: one stray "30 physics ticks" sentence remains (F829-R2-04) |
| R1-08 P2 metrics accounting/fingerprints | **CLOSED** | Creations vs rewrite-bytes instrumented at actual paths, raw per-frame series persisted, instrumented-total cross-check (sabotage → red), fingerprints (agp_commit/lock/assets) on every report, missing-observation ⇒ not complete ⇒ fail |
| R1-09 P2 vendoring boundary | **CLOSED** (code) | `patches/` deleted, stock iced_widget 0.14.2 from crates.io verified sufficient, zero path deps (metadata), lock regenerated, relocation re-verified. Docs: three stale "vendored patch" statements remain → F829-R2-03 |
| R1-10 P3 health/facts | **CLOSED with nits** | fmt clean, zero warnings, unreachable score assertion restored (tests assert per-squash score payloads), counts reconciled to 90. Stale-wording nits → F829-R2-04/05 |
| R1-11 P2 3D shared scene/identity contract | **CLOSED** | Squash fully on the G-base contract: session-scoped object IDs, spawn/transform/destroy deltas, renderer consumes `project_instances` of the shared scene; independent-consumer replay reconstructs + edits the 3D scene; stale-across-restart ID rejection asserted; delta-disabling mutation → red |

## New findings

### F829-R2-01 — P2: `ag2d-breakout` headless / window-fallback path can never pass (F07 sibling path missed)

The batch-3 F07 repair wired the per-pixel golden-frame oracle only into
the WINDOWED capture path (`BreakoutApp::capture_keyframe`, breakout.rs:820).
The offscreen path (`capture_keyframes_offscreen`, breakout.rs:593) — used
by `--mode headless` AND by the windowed run's documented offscreen fallback —
writes keyframes without any oracle comparison, while the pass predicate
(breakout.rs:364) unconditionally requires non-empty `golden_frame_checks`.
Result: on the current frozen commit, `--case ag2d-breakout --mode headless`
deterministically exits **1** with every gameplay fact green (reproduced;
report in `ag2d-breakout/`) — the headless keyframe hashes are in fact
bit-identical to the frozen goldens, so this is a pure gate-wiring
defect, not a content defect. Sibling asymmetry: `ag3d-squash` headless runs
the oracle fine (exit 0). The case's own usage string advertises
`--mode auto|window|headless`, R1's replay methodology ran breakout
headless, and the plan allows per-case headless modes — the invariant-level
fix for F07 did not cover this sibling path.

Repair: run `frame_oracle::compare_golden_frame` inside
`capture_keyframes_offscreen` (identical code to the windowed path; content
is already bit-exact), or explicitly reject non-window modes with exit 2
like the iced case does. Then re-run breakout headless + windowed.

### F829-R2-02 — P3: `independent-consumer` A-side depends on a prebuilt release binary (not self-contained)

`consumer.rs:315` runs `target/release/graphics-probe.exe` from whatever
state the tree is in, while B is freshly built in-case. Demonstrated on the
frozen commit: the case exited 1 solely on
`fingerprints.agp_commit 1fc3f61164b0 vs 95ccfcc5c18a` — the worktree release
binary was one commit (docs-only E14) stale; after
`cargo build --locked --release --bin graphics-probe` the same case exits 0.
Following README order (build release before probing) masks this, but the
in-code comment claims "self-contained: no reliance on whatever a previous
run left in scratch" while relying on a worse out-of-band artifact. Cannot
false-green (the fingerprint only diverges), so P3. Repair: build the
A-side binary in-case (or run A via the freshly built B-tree binary from the
original tree after an in-case build), or enforce freshness.

### F829-R2-03 — P3: three stale "vendored iced_widget patch" statements contradict the F09 de-vendoring

- `research/integration-handoff.md:96-98` (IH-05 "Delivered"): "the only
  cross-repo artifact is the VENDORED iced_widget patch with
  `patches/iced_widget/PROVENANCE.md`" — that tree no longer exists; the same
  document's dependency table (line 136) correctly says STOCK. Internal
  contradiction.
- `research/spec-delta/auto-graphics.project.md:27` (SD-01): "iced_widget
  patch vendored with provenance".
- `research/spec-delta/auto-graphics.integration-contract.md:22-23` (SD-03,
  IH-05): "vendored provenance-tracked patch".

These are exactly the F10-class "spec proposals must describe only verified
current facts" misses; SD-01/SD-03 cannot be deposited with this wording.

### F829-R2-04 — P3: `sample-manifest.md` §2.6 stray "30 physics ticks" sentence

Three lines below its own dated correction ("spawn every **60** physics
ticks (= 0.5 s at 120 Hz)"), the binding "Spawn cadence oracle" still says
"one mob every 30 physics ticks (0.5 s)". The frozen trace and all tests use
60. Missed F07 sub-item; one-line fix.

### F829-R2-05 — P3 (trivial): two stale counts in docs/comments

- `tests/squash_contract.rs:8` header comment: "47 events: 23 spawns, 3
  squashes, 1 death, 20 despawns" — the re-frozen (F03) trace actually has
  45 events (22 spawns, 3 squashes, 1 death, 19 despawns); the test asserts
  dynamically so nothing false-passes, but the comment misstates the frozen
  contract.
- `research/result.md` contains an intermediate "85 tests (lib 23 … iced
  seam 4)" line that contradicts its own final "90 tests" line two
  paragraphs later (actual measured: lib 24, cache 3, iced seam 5, total 90).

## AC disposition (AC-01..AC-08 work-owned; AC-09 review; AC-10 merge)

| AC | R2 result | Basis |
|---|---|---|
| AC-01 | **PASS** | own workspace/lock; closure zero path deps (metadata-verified); 30 hash sets verified; relocation A→B replay green (after release rebuild, F829-R2-02 noted); root diff empty outside AGP; no reparse points; stock iced_widget provenance documented |
| AC-02 | **PASS** | one device/identity foundation; cache ownership invariants closed (F04); 3D on shared scene/identity/delta contract (F11); gpu-foundation + shared-runtime + risk-slice green |
| AC-03 | **PASS** (windowed) + mode-matrix defect | frozen-profile gameplay bit-exact, audio chain 210/210, resets clean, windowed green; headless mode red by gate wiring (F829-R2-01) — does not invalidate the windowed evidence but must be fixed |
| AC-04 | **PASS** | move/stomp/side/fatal-retry all evidenced; full-payload trace bit-exact; detector channel; facing/animation; music real; shadow/retry-overlay frames per-pixel gated; original-engine baseline captured per decision item (a) |
| AC-05 | **PASS** | lifecycle teardown oracle now observes real teardown (witness, three-phase sampling, awaited cancellation); stale/late/restart controls green; R1's leak injection permanently red |
| AC-06 | **PASS** | honest CPU metrics + raw series + fingerprints; sabotaged accounting/verifier → red; GPU timing explicitly unavailable |
| AC-07 | **PASS** | real iced window: real click through real event path, same-device functional + architectural proof, DPI/resize/exit, composite gate discriminates (blank-blit → red), second-device gate fires |
| AC-08 | **PASS** (SD wording fix pending) | P0–P2 done / I1–I3 explicitly pending; IH-01..05 owners/gates/delivered/missing all present; only the IH-05 vendoring wording (F829-R2-03) must be corrected before deposition |
| AC-09 | **PASS** (this review) | all AC/SD independently re-verified on the frozen commit with regenerated counterexamples |
| AC-10 | pending merge-owned | unchanged: frozen-commit full AGP suite/probes + consumer relocation + handoff gates + Spec deposition + archive/cleanup |

## Spec delta verdicts

- **SD-02 (prototype-profile): VERIFIED** — every claimed fact matches
  observed behavior, including the R1-flagged ones: "no owned worker threads
  for the runtime" now coexists with "the iced spike's input fallbacks use
  short-lived helper threads (recorded)" (iced_spike.rs:903/986 spawn
  short-lived input helpers); composite claims describe the in-band counter
  method with the GDI limitation recorded; foreign-bind wording states the
  wgpu-27 API reality (not rejected) with the architectural guard. Target
  path/metadata correct.
- **SD-01 (project): NEEDS WORDING FIX** — scope/entries/I1–I3 pending all
  correct; line 27 "iced_widget patch vendored with provenance" is false
  post-F09 (F829-R2-03). One-line correction; then depositable.
- **SD-03 (integration-contract): NEEDS WORDING FIX** — IH-01 verified
  facts (composite method, same-device proof, stock iced_widget, permanent
  audit test) are accurate; IH-05 "vendored provenance-tracked patch"
  contradicts the de-vendored reality (F829-R2-03). One-line correction;
  then depositable.

## R2 decision items

**(a) Godot original-engine character-level ground truth — RULED ACCEPTABLE
AS THE SOURCE-ORACLE CONTRACT; character-level engine comparison recorded as
explicit deferred debt (non-blocking).** Reasons: (1) the Plan's oracle
rules (§6) mandate frozen-source-derived oracles and forbid the
implementation defining its own fidelity — they do not mandate
original-engine runs (the planning round itself only read sources); the
delivered chain (independent Python reference + frozen source constants +
bit-exact replay + an original-engine trajectory/semantics baseline)
satisfies oracle independence; (2) the engine capture that DOES exist
confirmed death/retry semantics, rest-height convention, and jump-apex
agreement within ~0.1 m, and it surfaced the honest divergence (real Jolt ⇒
0 squashes vs the approximation scenario's 3) — exactly what a ground truth
should do; (3) the character-rendering blocker is an environment constraint
(three documented headless import attempts), not a silently dropped check,
and METHOD.md defines the completion path. Condition: the deferral must be
registered in `docs/plans/KNOWN-DEBT-AND-RISKS.md` with the METHOD.md
pointer, and must not be paraphrased as "engine-verified fidelity" anywhere.

**(b) Content golden frames provenance (implementation-derived) — RULED
ACCEPTABLE.** The frames are declared as bit-exact canonical captures
(manifest.json provenance field), sha256-pinned, per-pixel gated with
corruption-discrimination verified; they function as regression anchors,
while fidelity is established one layer down by the independent reference +
frozen constants + engine trajectory baseline. This separation is documented
and honest; accepting it does not let the implementation define its own
correctness because semantic fidelity is anchored independently of the
frames.

## Routing / pending closeout

- Outcome **needs_fix**, narrowly scoped to F829-R2-01 (P2) +
  F829-R2-02..05 (P3). All R1 findings F829-R1-01..11 are verified CLOSED
  with discrimination evidence; do not reopen them. The affected surfaces
  are: T-04 (breakout offscreen oracle wiring), T-07 (consumer A-side
  self-containment), T-09/SD-01/SD-03/handoff/manifest (wording). A small r3
  fix batch + targeted confirmation of F829-R2-01..05 closure (re-run
  breakout headless+windowed and independent-consumer) is sufficient; no
  full acceptance re-run is required.
- Status stays `executing`; T-10 remains the final review gate (R3
  confirmation, then T-10 pass); T-11 merge-owned gates unchanged
  (frozen-commit AGP suite + probes + consumer relocation + migration
  handoff authorization + Spec deposition of the corrected SD-01..03 +
  archive + wt-guard cleanup).
- This review modified nothing in the implementation tree (verified clean
  before and after); injection copies were deleted after their mutations
  were logged above; all raw outputs are under this directory.
