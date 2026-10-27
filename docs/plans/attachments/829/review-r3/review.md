# PLAN-829 — targeted confirmation re-review R3

Date: 2026-10-21 (Asia/Shanghai). **Outcome: PASS** (all F829-R2-01..05
closed; one pre-existing P3 robustness observation recorded as debt; one
spot-check live rerun environment-blocked but covered by code identity +
R2-recorded evidence). Scope: closure confirmation of R2 findings only +
adjacent-gate no-regression. Reviewer: independent R3 agent; no worktree
file modified (verified clean before and after); evidence only under this
directory; AGP-scoped commands only, no root tiers.

## Baseline

- Worktree `D:/autostack/.wt/lang-829/auto-lang`, branch `plan-829-dev`.
- HEAD `077de3b6697c69053c24f4067bf73188ca90ab62` (claimed 077de3b66; matches).
- Prior reviewed commit `95ccfcc5c18a6c4a3051f80ee38ab19bf05d67f7`.
- Tree clean at entry and exit (`git status --short` empty, 0 lines).
- Diff vs base `fdb1fad2ce`: AGP-only (verified; no files outside
  `experimental/autographics-prototype/`).
- r3 commit `077de3b66` touches exactly 9 files: `breakout.rs` (+21, offscreen
  oracle), `consumer.rs` (+28/-7, A-side in-case build + note), docs
  (evidence-notes E15, result.md, sample-manifest, integration-handoff,
  SD-01, SD-03), `tests/squash_contract.rs` (comment). Iced path untouched:
  `git diff 95ccfcc5c..077de3b66` over `src/host/iced_spike.rs`,
  `src/host/cases/iced_shared_device.rs`, `src/host/gates.rs` is EMPTY.
- AGP `Cargo.lock` sha256 unchanged from R2: `6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`.

## Per-finding closure verdicts

### F829-R2-01 (P2) breakout headless/fallback golden gate — CLOSED

- Code: `capture_keyframes_offscreen` (breakout.rs:637-657) now runs
  `frame_oracle::compare_golden_frame` with identical semantics to the
  windowed path (same has_golden guard, same (name,mean,p99,ok) recording,
  same (-1,-1,false) failure on integrity miss). Pass predicate unchanged
  (requires non-empty all-ok checks), so a headless pass is only reachable
  with the oracle actually executed — not a vacuous exit 0.
- Runs (all exit 0, reports under this dir): breakout `--mode window`
  (`breakout-window/`), breakout `--mode headless` (`breakout-headless/`),
  squash `--mode window` (`squash-window/`), squash `--mode headless`
  (`squash-headless/`). All four reports `exit: pass` with 4 keyframes each;
  breakout windowed ran a real window (mode "window 1800x1200 scale 2").
- The 4-run matrix is exactly the per-mode verification adjustment the E15
  diagnosis promised; squash both-modes re-run confirms no regression on the
  sibling game.

### F829-R2-02 (P3) independent-consumer A-side self-containment — CLOSED

- Code: consumer.rs:311-330 builds the A-side reference binary in-case from
  the original tree into isolated `CARGO_TARGET_DIR`
  (`scratch/829/consumer-a-target`) before the reference run; B-side remains
  an in-case build of the relocated copy; the "self-contained" comment now
  describes this honestly.
- From clean state: deleted worktree-root `scratch/829/consumer-a-target`
  and `scratch/829/relocate` (note: the case's scratch lives at the
  WORKTREE ROOT `scratch/829/`, not AGP-local), then:
  - run 1 (cold A-side build): exit 0, 66 s; `relocation.ok=true`,
    `mismatches=[]`; fingerprints.agp_commit `077de3b6697c` on both sides.
  - run 2 (incremental A-side): exit 0, 34 s; same zero-mismatch result.
  - Verified `scratch/829/consumer-a-target/release/graphics-probe.exe`
    exists — the A-side binary is genuinely built in-case. The R2 false-red
  precondition (stale prebuilt `target/release` binary) no longer exists.

### F829-R2-03 (P3) stale "vendored patch" wording — CLOSED

Sweep of `src/ tests/ research/ README.md Cargo.toml` for
`vendored|vendoring|patches/iced_widget`: all remaining mentions are
historical/descriptive and accurate (consumer.rs:87 "the vendored patch was
removed"; handoff:98 "the vendored iced_widget patch was removed after
stock … verified green unpatched"; SD-01:28 "de-vendored …"; SD-03:23 "r1
F09 de-vendoring; no provenance-tracked patch remains"; result.md E15
narrative; support-profile.md:24 correctly attributes `[patch.crates-io]`
to the PRODUCTION root while stating AGP is stock with zero path deps). No
contradiction of the stock-iced_widget reality remains.

### F829-R2-04 (P3) manifest §2.6 spawn cadence — CLOSED

`sample-manifest.md:385` now reads "one mob every **60** physics ticks (=
0.5 s at 120 Hz)" with a dated 2026-10-21 correction note; the stray
"30 physics ticks" sentence is gone (grep confirms zero remaining
occurrences of "30 physics ticks" outside the correction note).

### F829-R2-05 (P3) stale counts — CLOSED

- `tests/squash_contract.rs:8` header: "45 events: 22 spawns, 3 squashes,
  1 death, 19 despawns" — matches the frozen trace exactly (verified by
  counting `testdata/squash/golden_squash_trace.json` events: 22/3/1/19=45).
- `research/result.md` AC-04 row says "45/45 events: 22 spawns, 3 squashes,
  1 death, 19 despawns"; the intermediate count line now reads the measured
  90 (the "85 tests" text survives only inside the E15 narrative quoting
  what earlier text was corrected).

## Two-failure diagnosis (E15) — VERIFIED

`research/evidence-notes.md` E15 exists and meets all three required
properties: (1) it names the invariant (oracle evidence producer must run
in EVERY advertised render mode, not just the one the repair replayed);
(2) it explains why both earlier stages missed it — work replayed R1's
counterexample only through the wired (windowed) path, R1 tracked the
sibling-path demand at the predicate level without decomposing per mode;
(3) the detection adjustment is actually applied — the r3 verification
matrix (breakout + squash × window + headless, all exit 0, reproduced by
this review) implements the per-mode rule. No contradictions with observed
behavior.

## No-regression spot checks

| Check | Result |
|---|---|
| `cargo test --locked` | exit 0, **90/90** (lib 24, breakout 11, cache 3, iced_seam 5, prototype_contract 16, squash 14, threed 10, verifier 7); zero compiler warnings |
| `cargo fmt -- --check` | exit 0 |
| probe `verifier-negative-controls` (headless) | exit 0, all 5 gates positive+negative fired (`negative/`) |
| probe iced-shared-device windowed — live rerun | **ENVIRONMENT-BLOCKED in this session** (see below); no-regression established by code identity + R2 evidence |
| `AGP_SABOTAGE_ICED=blank_blit` live rerun | environment-blocked (same precondition); discrimination unchanged by code identity |

**Iced windowed rerun — environment-blocked, root-caused, non-regression
established.** Five attempts (positive ×3, sabotage ×1, diagnostic ×1) all
stalled in phase 1 (initial keyframes captured; click never observed).
Diagnosis: (1) two ZOMBIE `graphics-probe.exe` windows with the identical
title "AGP iced-shared-device spike" were found alive from earlier
timeout-killed runs — `FindWindow` returns the first (dead queue), so the
case's own clicker could never succeed; after `taskkill /F` of both zombies,
(2) a further clean attempt STILL hung, with the desktop showing active user
foreground apps (Chrome/Feishu/etc.); the window exists with the exact
title on the interactive desktop, but input delivered from this agent
session does not register — the same E14-documented agent-session
focus/input flake, now aggravated by user activity. Decisive code fact:
`git diff 95ccfcc5c..077de3b66` over `iced_spike.rs`,
`cases/iced_shared_device.rs`, and `gates.rs` is **empty** — the iced gate
is byte-identical to the code R2 verified live (windowed exit 0 with
composite counters advancing 1→501→504; blank-blit sabotage exit 1 with
composite.ok=false). The windowed iced rerun is re-queued as a merge-owned
gate on the frozen commit in an interactive session (it was always a
merge-owned rerun item; R2's recorded evidence covers the code-identity
argument).

## New findings

### F829-R3-01 — P3 (pre-existing, not a regression): iced phase-1 waits unbounded for the click

`canonical_step` phase 1 (`iced_spike.rs:837-843`) transitions only when
`click_observed`; the two clicker threads are bounded (~45 s of retries
total) but on exhaustion the event loop waits FOREVER instead of failing
with a reason. Any environment where input delivery fails (this session's
flake; locked desktops; CI without interactive station) turns the case from
fail-fast into a hang. Unchanged since long before R2 (R2's successful runs
never exhausted the retries), so recorded as debt, not an R2-closure gap.
Recommend: when both clicker threads exhaust (or after a bounded phase
timer), transition to a fail phase with `click_ok=false` so the case exits 1
with a diagnostic. Register in KNOWN-DEBT-AND-RISKS.md.

No other new findings. In particular: the r3 commit introduces no fmt/warning
/test-count drift, no lock/dependency change, no touch outside AGP, and the
offscreen-oracle/consumer refactors are minimal, surgical, and verified by
the reruns above.

## Recommended plan status / next

- All F829-R2-01..05 **CLOSED**; R2's routing condition (small r3 batch +
targeted confirmation) is satisfied. Recommend `status: execution_done`,
T-10 (final review) **PASS** at frozen commit `077de3b6697c69053c24f4067bf73188ca90ab62`.
- T-11 merge-owned gates (unchanged, all pending and must run on the frozen
  commit): full AGP suite + all 10 probe cases incl. **iced-shared-device
  windowed in an interactive session** (covers the environment-blocked
  rerun; ensure no stale same-title zombie windows first) + blank-blit
  sabotage spot re-check; real consumer relocation replay; migration
  handoff authorization; Spec deposition of SD-01/02/03 (wording now
  verified correct) + ledger; register F829-R3-01 and the character-level
  Godot engine comparison (R2 decision item a) in
  KNOWN-DEBT-AND-RISKS.md; archive; wt-guard clean + worktree/branch
  cleanup.
- Do NOT reopen any F829-R1-01..11 or F829-R2-01..05 item.
