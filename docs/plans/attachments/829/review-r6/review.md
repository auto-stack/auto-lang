# PLAN-829 — independent final re-review R6

2026-10-21, Asia/Shanghai. **Outcome: needs_fix**. Scope: final, explicitly
requested again by the user. Reviewer: original R1/R4 review chat, independent
of implementation; no new agent/chat was created. R5's labels were treated as
claims and reconciled with source, durable inputs and executable controls.

## Frozen baseline

- Existing tree `D:/autostack/.wt/lang-829/auto-lang`, branch `plan-829-dev`.
- HEAD **b84c6d91e83a00d4101e6e42403820127516c061**; base
  **fdb1fad2ce7afa124adfa3080323496df384e670**. Entry and final source checks
  clean; no implementation edits during review. A clean status does NOT
  include ignored files: a mandatory runtime manifest is absent from HEAD
  (F829-R6-03 below).
- Entry Plan r1, status reviewed, frontmatter10/11; several task checkboxes
  and the current-state paragraph still reflected R4. Entry Plan SHA256
  `b834987401e260823706a6dd328d3e920975986149ff866a00599e4b833738fd`.
- AGP lock unchanged: SHA256
  `6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`;
  wgpu27.0.1, winit0.30.13, iced/iced_wgpu0.14.0, stock iced_widget0.14.2.
- Specs read/reconciled: overview, goals/GOAL-008, UI overview,
  repository-boundaries/project. Three proposed auto-graphics Spec paths still
  absent from canonical Specs. Exact inputs and local runtime manifest hashed
  in [input-sha256.txt](input-sha256.txt); SD proposals frozen as
  `spec-delta-*.md` beside this report. No canonical Spec or ledger published.

## Executed checks and limitations

Reproduction entry: [replay.ps1](replay.ps1) (frozen-SHA guard; optional `-Live`
for the bounded windowed attempt). Default replays source, pixel and committed
delivery controls without changing implementation.

All implementation commands ran in the existing tree. `M` =
`experimental/autographics-prototype/Cargo.toml` (AGP).

| Check | Actual result |
|---|---|
| `cargo test --manifest-path M --locked` | **97/97**, exit0, no compiler warnings (lib28 + breakout11 + cache3 + iced5 + prototype16 + squash17 + threed10 + verifier7) |
| `cargo build --manifest-path M --locked --lib --bin graphics-probe` | exit0; actual frozen library linked by independent probes |
| `cargo fmt --manifest-path M -- --check` | exit0 |
| `git diff --check BASE...HEAD` | preserved upstream FiraMono license whitespace exception only ([excerpt](diff-check.txt)) |
| Independent public-API behavior replay | old R4-01..04 examples **closed**; new source-height mismatch reproduced ([source](behavior-probe.rs), [stdout](behavior-output.txt)) |
| Official `ag3d-squash`, canonical, headless | exit0; [report](squash/report.json.txt), new retry frame accepted using the local ignored manifest; does not establish source spawn-height fidelity or clean delivery |
| Independent old/new retry image comparison | **3563 changed pixels**, bbox **[195,408,295,474]**; exactly R5's claimed localized player→mob correction ([stdout](pixel-oracle-output.txt)) |
| Actual Godot4.4.1, unchanged frozen Main.gd + Mob.gd | exit0; **four trials actual mob y=0**, path local y1/follower local y0/global y1 ([script](source-spawn-probe.gd), [stdout](source-spawn-stdout.txt), [status](source-spawn-status.txt)) |
| Exact current screenshot analyzer | real normal screenshot passes, real transparent screenshot rejects; 3D padding bleed and shifted layout both erroneously pass ([probe](pixel-oracle-probe.rs), [function copy](analyzer.rs), [stdout](pixel-oracle-output.txt)) |
| HEAD-only delivery export + committed frame-oracle consumer | manifest absent, compareNone; adding only local ignored manifest in review fixture makes comparison Some(0,0,true) ([source](delivery-oracle-probe.rs), [stdout](delivery-output.txt)) |
| Original current Iced/windowed positive | **90s supervisor deadline**, initial offscreen frames only, no composed screenshot/report returned ([status](iced-positive-status.txt)). Own process tree terminated. No new rendering defect is inferred from that timeout; current live positive/negative acceptance remains unverified. |

The Godot fixture is a **minimal source-emitter test**, not a full imported
Godot game run: frozen Main.gd/Mob.gd/ScoreLabel.gd are copied unchanged; the
Path3D/Curve3D/PathFollow3D values match Main.tscn; actual
`Main._on_mob_timer_timeout` calls the actual `Mob.initialize`. Mesh/audio/
physics imports do not affect the position value passed to that initializer.
Engine version prints `4.4.1.stable.official.49a5bc7b6`, matching the existing
research reference. Four seeds distinguish the coordinate property from RNG.

The pixel test uses byte-preserved real same-machine work captures as **bound
validator inputs**, with their hashes recorded. The current private analyzer
was extracted without logic changes (only made public in a review module),
compiled against current Cargo JSON-resolved dependencies. This is an executed
oracle counterexample, not a claim that the current renderer itself emitted
the deliberately corrupted images. Original + transparent controls establish
the intended positive/negative domain; the faulty inputs exercise the exact
function called by `Message::CompositeShot` and `composite_content_facts`.

No root t/tv/tt/tb/tu/taa/tf/docs_gen, new full relocation rebuild, merge,
archive or cleanup was performed. R5's same-SHA consumer and unaffected
runtime receipts are historical evidence; they are not new R6 executions.

## Findings

### F829-R6-01 — P2: source sample spawns at local y0; AGP hardcodes y1

**AC/task:** AC-01/04/08; T01/T05/T06/T07/T09. This is an existing source-fidelity
requirement, not a new scale/profile promise or a Jolt approximation issue.

**Path and existing rule:** frozen Main.tscn has SpawnPath.position.y=1, but
its child SpawnLocation local position and curve points have y0. Main.gd:37
passes **`mob_spawn_location.position`**, not global_position, to
Mob.gd.initialize; it adds the initialized mob directly under Main.
AGP `games/squash/sim.rs:51,404,412` instead uses `SPAWN_Y=1` and labels it
as source-exact. `research/3d-feasibility.md:44` repeats this mistaken rule.

**Bound entry/result:** original-engine source fixture runs those unchanged
source methods for four seeds. Every trial prints:

```text
path_y=1 follower_local_y=0 follower_global_y=1 actual_spawned_mob_y=0
```

Independently linked current AGP emits its first Spawned event at
`Vec3(14,1,-0.35427094)`, actual_y=1. Expected source y=0. The global-position
value1 is a legitimate control demonstrating exactly how the implementation
misread the parent transform. No mesh import or numerical physics assumption
is needed to distinguish these paths.

**Cause established:** source local/global coordinate confusion carried into
the implementation and its Python/golden expectations. The current golden
passing proves consistency with that implementation-derived expectation, not
fidelity to the frozen source. It affects visible model height, body/detector
encounters and the replay scenario's expected gameplay, so debt registration
cannot exempt it.

**Closure:** derive spawn positions from the actual frozen call path, correct
the implementation/research/oracle together under that source expectation,
and verify actual source vs AGP spawn values, meaningful stomp/side/death/retry
controls and both render modes. Recheck related node-origin/sphere-center and
parent/local coordinates as part of the repair; this probe establishes spawn
height specifically and does not assert untested errors in those other paths.
Do not just retune the test coordinates or re-freeze the wrong height.

### F829-R6-02 — P2: new composed-pixel gate misses 3D clip bleed and layout displacement

**AC/task:** AC-05/06/07; T07/T08/T09. Location:
`host/iced_spike.rs:834..866,960..990`; consumed by the actual screenshot
message handler and the formal case's `composite_content_facts` admission.

The function claims both views at layout-derived expected rects with clipping.
In fact it **searches the observed image** for the first dark2D row, accepts
that row as the correct top, and only checks the **2D clear color** when
looking for view pixels outside rects. Sky-colored3D pixels outside the3D
rect are invisible to that clip check.

**Bound inputs and exact results:** start from the real 2000×1600@2x positive
window screenshot. Current source layout gives first view physical y98,
height640, gap16, padding20; its actual source image is accepted.

1. Change only both padding strips alongside the3D view to its source sky
   color (165,167,171): **25,600 pixels outside the view change**. Expected
   clip=false. Actual `ok=true`, `clip_ok=true`, `pad=0.000`; all other facts
   remain true. [Corrupt input](pixel-3d-padding-bleed.png).
2. Leave UI controls unchanged; shift both complete graphics views down80
   physical pixels on the same fixed-size window. Expected position failure
   (top178 vs expected98). Actual `views_top=178`, every fact true, `ok=true`.
   [Corrupt input](pixel-shifted-views.png).
3. Original real positive remains `ok=true`; actual transparent negative is
   now `ok=false` (view2d/3d/nonblank false). Thus the R4 transparent example
   is discriminated; the claimed clipping/position invariant is not complete.

These are valid Screenshot byte inputs from representative wrong composite
outputs. The experiment executes the exact analyzer, not an artificial flag
or a modified PASS print. The new live-window sequence itself did not finish
this round, so this report does not mislabel these as fresh full-runner
executions. The downstream admission accepts `facts.ok()` and has no separate
rect/clip check that would reject them.

**Closure:** independently anchor expected UI/view geometry from the actual
layout/viewport, and observe out-of-bounds content for **both** views. Add
normal/transparent,3D bleed, actual position/crop errors and resize/DPI
controls through the same consumed screenshot path; retain a completed live
positive and negative on the new frozen code. This remains the existing
V06/V08 bounded probe requirement, not a request for a generic vision checker.

### F829-R6-03 — P1: required golden-frame manifest is missing from the reviewed commit

**AC/task:** AC-01/03/04/06/08; T01/T07/T09.
`testdata/golden_frames/manifest.json` is a required runtime input of
`host/cases/frame_oracle.rs`, but `.gitignore:161 (*.json)` hides it.

```text
git ls-files -- <manifest>                   # empty
git check-ignore -v <manifest>               # .gitignore:161:*.json
git cat-file -e HEAD:<manifest>              # nonzero, path absent in HEAD
```

Raw receipts: [ignore rule](manifest-ignore.txt),
[HEAD error](manifest-head-error.txt), [status](manifest-head-status.txt),
[empty tracked listing](manifest-ls-files.txt). The file's actual local bytes
are preserved as [local-untracked-manifest.json.txt](local-untracked-manifest.json.txt)
and hashed with the input package.

**Actual delivery consumer test:** `git archive HEAD experimental/autographics-prototype`
exported a fresh delivery fixture (not a second worktree). Compile the exact
committed `frame_oracle.rs` with its manifest-root pointing to that export.
With the official newly rendered retry PNG, the exported consumer reports:

```text
manifest_exists=false golden_declared=false compare=None
```

As a legitimate positive control **only in the disposable review fixture**,
add the ignored local manifest: the same consumer reports
`golden_declared=true compare=Some((0,0,true))`. No implementation file was fixed.

The games condition comparisons on `has_golden` and require nonempty successful
frame checks; a tracked-only delivery cannot satisfy that acceptance condition.
This round executes the exact reader and verifies the committed predicate; it
does not claim a cold full-game run from the export. Current97 tests and the
local gameplay PASS use a file outside the reviewed Git revision. The V09
consumer's copy of the whole working directory can carry ignored inputs, and
its shared-runtime-only replay does not exercise this golden dependency.

**Closure:** include every necessary oracle/runtime input in the versioned
delivery closure, make the gate's required input inventory checkable, and
replay both advertised games from an export containing **only committed files**.
Audit other required ignored files/receipt references. Rebind review evidence
to the resulting complete commit; working-directory cleanliness alone is not
the release input boundary.

## R4/R5 closure and oracle disposition

- **R4-01 closed** on the source identity path: actual canonical death leaves
  four mobs, no projected Player, independent log replay agrees. Its new retry
  capture differs from R4's exact old capture by3563px in the single stated
  bbox. The source rule queue_free⇒no player image establishes that localized
  update; **this frame repair is accepted as honest**, not rejected as oracle
  manipulation. F6-01 is a different source-coordinate rule still wrong.
- **R4-02 closed**: public press-edge machine now returns RetryPressed for
  dead Space and leaves no stale jump; current real host passes dead/visibility.
- **R4-03 closed within its rigid-yaw invariant**: exact earlier bound pair
  now agrees false/false; source derivation and permanent joint-yaw tests pass.
- **R4-04 closed**: rotation changes snapshots; q and -q give equal snapshots.
- **R4-05 transparent-specific classifier is effective** on real bound pixel
  controls; new live screenshot admission remains unverified in this session,
  and the broader clip/placement claim is partial via F6-02.
- Previous cache/lifetime/timing/stock-dependency repairs are preserved; no
  return to their old examples is alleged. R1-F07's source-oracle and complete
  replayability boundary remains partial via F6-01/03.
- R5-01 unbounded wait/R5-02 runbook/R5-03 SD wording remain historical notes;
  no blanket process-kill instruction was followed. Only R6-owned timed-out
  process trees were stopped. The90s result is an evidence limitation, with no
  invented environmental root cause or product-hang diagnosis.

## AC/SD review and routing

| AC | Result and evidence boundary |
|---|---|
| AC-01 | fail: source inventory misread height; required delivery input absent from commit (F6-01/03) |
| AC-02 | scoped pass: core identity/rotation and previous GPU/resource foundation retained; no new defect found there |
| AC-03 | implementation evidence retained; complete delivery replay partial due missing manifest; unrelated2D rules not reopened |
| AC-04 | fail: source height mismatch despite local trace/content PASS (F6-01) |
| AC-05/06 | partial: prior lifetime/accounting work retained; clip/position false-green and unbound required input remain |
| AC-07 | fail/partial: current exact analyzer misses required controls; current live positive timed out without a verdict (F6-02) |
| AC-08 | partial: I1–I3 pending/IH owners remain appropriate; delivered/profile claims need correction |
| AC-09 | fail: explicit independent review completed, three open in-contract findings |
| AC-10 | merge-owned pending, not attempted |

SD-01's module/boundary identity remains suitable, but complete reproducible
entries cannot be claimed until F6-03 closes. SD-02's sample/profile/replay
facts need F6-01/03; SD-03's verified per-widget clipping needs F6-02 and its
new screenshot-method wording reconciled before deposition. Freeze copies are
**not approved for publication**. New targets remain the three
`docs/specs/auto-graphics/...` files, supersedes[], GOAL-008 verified and retained.

Return to work in the **same tree**. Preserve T02/T03/T04 complete; reopen or
keep open T01/T05/T06/T07/T08/T09/T10. State executing, **3/11**; T11 pending.
The earlier checklist/summary mismatch is normalized by this current disposition,
without deleting R5's history or undoing its valid specific fixes.

Multiple-review diagnosis must now extend E16 with **actual source local/global
coordinates**, **both views' clipping/independently anchored geometry**, and
**committed-delivery vs ignored working files**. Test the closing invariant
on the real producer/consumer, retain unchanged positive controls, then submit
a complete committed baseline. No semantic Plan requirement was changed by review.

Pending closeout still includes repaired-code final acceptance; merge-owned
frozen independent-workspace tests/probes, successful Iced live positive/negative
and real committed consumer relocation; migration/write-surface handoff;
canonical Spec/ledger reconciliation; archive and guarded cleanup. This review
did not repair implementation, merge, archive, remove branches/worktrees, publish
canonical Specs/ledger, or message another chat.
