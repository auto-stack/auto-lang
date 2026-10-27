# PLAN-829 final review R8 — needs_fix

2026-10-21. Independent acceptance review in the original review chat, separate
from implementation. No agent/chat created. Review scope: final; plan revision 1.
Baseline: `814f9bf01aa14eadcd9e851bc4cce7ac8caa360b`, branch `plan-829-dev`,
existing worktree `D:/autostack/.wt/lang-829/auto-lang`. Base:
`fdb1fad2ce7afa124adfa3080323496df384e670`. Tree clean at entry and exit.
Plan entry SHA256: `521D173A51B95F1AC80492DE96C49626057858E0B41CEF97365463D729D901F0`.

## Decision

Two findings require work. R6's original pixel examples are rejected now, but
R6-02's complete placement/clipping invariant remains partial. The source-height
implementation and committed delivery are fixed; the source-oracle documentation
still contains the old wrong height. R7's pass does not cover these holdouts.

## F829-R8-01 — P2: the screenshot gate misses independent 3D displacement and bottom clipping

- Existing rule: V-06 / AC-07 requires real composed views with correct clipping,
  DPI and resize; V-08 / AC-05/06 requires the actual validator to distinguish
  incorrect output. Tasks T-07/T-08/T-09/T-10. This is incomplete closure of
  F829-R6-02, not a new arbitrary-nesting or generic image-verification promise.
- Actual path: Iced `Message::CompositeShot` → `analyze_composite_shot` → stored
  `CompositeShotFacts` → `iced_shared_device::finalize`, which consumes `s.ok()`
  for each moment. The public function is the real consumed analyzer, linked
  directly from the Cargo-resolved current AGP library; no extracted replacement.
- Bound input: R6's durable **real** positive screenshot, 2000×1600 at scale 2,
  unchanged controls and 2D view. Expected 3D rect is x=[20,1980),
  y=[754,1394), derived from the fixed producer layout.
- Probe/command: [pixel-probe.rs](pixel-probe.rs), [replay.ps1](replay.ps1).
  Raw facts: [pixel-output.txt](pixel-output.txt). Hashes bind the exact inputs.

| Input | Mutation / independently expected result | Actual consumed facts |
|---|---|---|
| Original real composite | Accepted | `ok=true position=true clip=true` |
| R6 transparent capture | Rejected | `ok=false`, both views absent |
| R6 25,600-pixel side sky bleed | Rejected | `ok=false clip=false` |
| R6 both-view down-80 translation | Rejected | `ok=false position=false` |
| **3D-only down-80 translation** | Reject displacement; 3D moves to y=[834,1474), controls/2D untouched | **`ok=true position=true clip=true`**, 66,450 changed pixels |
| **Bottom sky bleed** | Reject 1960×80=156,800 sky-colored pixels at x=[20,1980), y=[1394,1474), wholly outside 3D rect | **`ok=true position=true clip=true`** |
| **3D-only down-80 with correct clipping** | Reject wrong placement even when excess below y1394 is removed | **`ok=true position=true clip=true`**, 37,490 changed pixels |

Result images: [3D displacement](3d-only-down80.png),
[bottom bleed](3d-bottom-sky-bleed.png),
[displacement clipped to the original rect](3d-only-down80-clipped.png).
All transformations occur after loading the real capture; changes are counted,
saved and fed into the real analyzer. The final clipped variant separates
placement from overflow. Both view-presence facts remain true in all three
invalid cases; f_sky remains 0.159 or 0.182, above the 0.03 threshold.

Established causes:

1. `iced_spike.rs:881` `position_ok` observes only the first mostly-2D-background
   row. Anchored sampling of the 3D rectangle checks broad presence, not that the
   3D view occupies its independently expected position. Moving 3D alone leaves
   that position fact unchanged.
2. `iced_spike.rs:1015` `clip_ok` checks gap, control band and side padding only.
   There is no sample of the lower outside region y≥v3_bot. Even the exact sky
   signature that the repaired side-padding check recognizes is ignored there.
3. Permanent controls move both views together and leak sky through the sides.
   They do not exercise independent per-view placement or the lower boundary.

Closing oracle: retain real positive and resize/DPI controls, the old rejected
inputs, and bind independent view placement plus the complete outside boundary
for both views. Reject the three attached invalid cases through the consumed
gate; demonstrate the repaired assertions distinguish actual weakening. Do not
remove the position promise, weaken source expectations or re-freeze these
corruptions as acceptable. The finding proves a validator defect; it does not
claim the current renderer itself emitted the corrupted images. No new full
live Iced case was executed in R8, and no UI/environment diagnosis is inferred.

## F829-R8-02 — P3: current source-oracle documents still teach the corrected mistake

- Existing rule: T-01 / AC-01 requires the frozen sample's numeric/behavior oracle;
  T-05/T-09 / AC-08/09 requires truthful research and delivery facts. R6-01
  explicitly required correcting the source interpretation and its oracle chain.
- `research/sample-manifest.md:206` still describes the spawn path as `at y=1`
  followed by `mob.initialize(spawn_pos, player_pos)`, omitting the decisive local
  `.position` call and the actual y=0. This is the active source-rule inventory,
  without a dated correction or superseded marker.
- `research/result.md:224` still says `spawn height y=1 resolved per frozen path`
  in its deviations/corrections summary. The dated r6 section elsewhere is
  correct, leaving contradictory active summaries.
- `research/3d-feasibility.md:74` claims the canonical run's death is at t=1402.
  Current committed trace has squashes at 995/1938/1990 and death at **2703**.
  This is presented as its current validation evidence, not a historical run.
- Evidence: [manifest excerpt](stale-source-manifest.txt),
  [result excerpt](stale-result-summary.txt), [feasibility excerpt](stale-feasibility.txt),
  [committed events](current-trace-events.json.txt).

Independent accepted control: the same frozen Godot Main.gd/Mob.gd fixture
executed again in Godot 4.4.1, four trials, all actual_spawned_mob_y=0, exit 0
([stdout](source-spawn-stdout.txt), [status](source-spawn-status.txt)). Current
public AGP driver emits (14,0,-0.35427094), agreeing with it
([behavior-output.txt](behavior-output.txt)). The generator and dated refreeze
are correct; no demand to restore old physics or modify frozen source/assets.

Correction: reconcile the active sample oracle and current summaries with the
local-coordinate source rule and the current trace, preserving history under
explicit dates. Audit related source-constant and current-evidence claims, not
only the files changed in the repair. Close with a documented consistency audit
against the unchanged source fixture/current trace; no heavy test tier needed.

## Verified closures, health and coverage

- **R6-03 CLOSED**. Manifest is in HEAD and not ignored. Both delivery tests pass.
  The committed required-input array has **19** paths; R7's 24-path receipt count
  is inaccurate and is not repeated as evidence here.
  R8 independently performed `git archive HEAD` → own export → `--locked
  --offline` build → **both games headless exit 0**, bit-exact trace and actual
  populated golden comparisons. Inputs are resolved from the exported
  `CARGO_MANIFEST_DIR`; only dependency compilation cache was reused. Fresh
  exported AGP compilation is recorded in [export-build.txt](export-build.txt).
  Reports: [Breakout](export-ag2d-breakout.json.txt),
  [Squash](export-ag3d-squash.json.txt); logs in the adjacent output files.
- **R6-01 code/generator/trace CLOSED; documentation partial** as F829-R8-02.
  Source y0 control and 17 squash tests pass, including full trace, spawn stream,
  stomp, death/retry and old rigid-rotation tests. Refreeze is accepted as an
  honest source-derived correction. The scenario-author jump-policy change is
  documented and is not a source constant.
- **R6-02 original inputs CLOSED; invariant PARTIAL** as F829-R8-01. Transparent
  negative still rejects. Do not label the full clipping/placement promise closed.
- R4-01/02/03/04 independently replayed: after death all six surviving objects
  are Mob in both live scene and changelog replay; dead Space retries with no
  stale jump; rigid-rotation pair false/false; snapshot records rotation and
  preserves sign-equivalent quaternion equality. [Probe](behavior-probe.rs).
- **45/45 targeted tests**: composite 5, delivery 2, squash 17, Iced seam 5,
  shared runtime 16. [Affected tests](targeted-tests.txt),
  [shared/seam tests](unchanged-contracts.txt). This is R8's actual count; the
  104-test total belongs to R7. No root whole tier or merge-owned full matrix ran.
- `cargo fmt --all -- --check` passes; repair diff-check b84..814 passes; no
  warning in targeted builds. Full base..HEAD diff-check retains one trailing
  space in the verbatim frozen FiraMono license, unchanged by the repair; not a
  new Rust-format or compiler defect. No stray debug code identified in repairs.
- All changes vs approved base remain inside `experimental/autographics-prototype`;
  no root workspace/lock or external path dependency changes, no reparse points.
  Lock SHA256 `6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`;
  wgpu 27.0.1 / winit 0.30.13 / iced and iced_wgpu 0.14.0 / stock iced_widget 0.14.2.
- Unchanged Breakout source/runtime, cache, lifecycle, measurement and production
  boundary evidence from R4/R6/R7 is reused **only within its earlier verified
  scope**: git b84..814 shows no changes to those implementations, dependency
  lock or test configurations. The source-height repair's effects on Squash
  replay/rendering were newly checked above. Old Iced end-to-end success does
  not certify the new gate; unresolved full live positive/negative remains pending.

## Acceptance and knowledge delta

| AC | Task / artifact / current evidence | R8 |
|---|---|---|
| AC-01 | T-01/T-09; export + tracked manifest; source-rule inventory | Partial: delivery fixed, F02 |
| AC-02 | T-02/T-03; shared contract 16 tests, unchanged GPU/cache inputs | Pass in declared profile, prior evidence bounded as above |
| AC-03 | T-04; committed-only Breakout probe + unchanged source evidence | Pass |
| AC-04 | T-06; source fixture, driver control, squash17, export game/render | Pass runtime; F02 research inconsistency separately recorded |
| AC-05 | T-07; shared/identity old controls accepted; invalid composed output accepted | Partial: F01 |
| AC-06 | T-07; unchanged measurement facts; validator cannot establish its claimed checks | Partial: F01 |
| AC-07 | T-08; consumed composed gate position/clip | Fail: F01; no new full live-case pass claimed |
| AC-08 | T-09; I1–I3 pending boundary retained, active summaries inconsistent | Partial: F02 |
| AC-09 | T-10; independent review of actual assertions | Fail / needs_fix |
| AC-10 | T-11 | Merge-owned pending |

Canonical inputs checked: specs overview, UI overview, repository-boundaries
project, GOAL-008 and Design45. Source hashes are frozen in this directory.
SD-01/02/03 proposals are copied verbatim here with hashes. They retain sound
experimental/production boundaries; they are **not approved for publication**
with the open acceptance gaps. SD-03's existing R5-03 stale counter-only/GDI
wording must be reconciled with actual screenshot validation after the repair;
it cannot claim full visible placement/clipping on the present code.
Metadata stays `supersedes_spec_components: []`, three exact new
`docs/specs/auto-graphics/{project,prototype-profile,integration-contract}.md`
paths, `touched_goals: [GOAL-008]`. No canonical Spec or live ledger changed.

## Route

`status: executing`, `current_step: 4/11`. T-02/T-03/T-04/T-06 complete;
T-01/T-05/T-07/T-08/T-09/T-10 open; T-11 pending. The entry Plan had reviewed/10
frontmatter and an R7 pass receipt while its current-status prose and several
task boxes still described R6/3 completed; this bookkeeping was reconciled to
R8 rather than treating either stale status as acceptance evidence.

Return to work in the same tree: fix both findings, extend E17's diagnosis to
independent per-view placement/all outside edges and current oracle consumers,
retain accepted controls, commit and request affected re-review. This review
does not authorize delaying an in-contract requirement or changing the oracle
to fit corrupt output. Historical closed repairs remain preserved.

After final pass: frozen AGP merge gates and live Iced positive/transparent
negative, real consumer relocation, migration/write-surface handoff clearance,
Spec/derived ledger deposition, archive and guarded cleanup remain pending.
No implementation fix, commit, merge, archive, worktree cleanup, cross-chat
message or root t/tv/tt/tb/tu/taa/tf/docs_gen was performed in R8.
