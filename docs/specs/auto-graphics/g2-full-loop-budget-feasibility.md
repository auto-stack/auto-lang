---
file: docs/specs/auto-graphics/g2-full-loop-budget-feasibility.md
source_plan: auto-ui/PLAN-011
plan_revision: 2
tags: [autographics, G2, budgets, feasibility, no_go_current_profile]
---
# G2 full-loop budget feasibility

The fixed sprites2d public profile does **not** satisfy the frozen complete G2
core in the two tested Auto candidates. This is `no_go_current_profile`, limited
to these candidates and inputs; it is not a proof that all Auto programs fail.
The research is delivered; a complete playable G2/M2 game is not established.

## Fixed consumer and complete denominator

The study consumes auto-ui0261ae9dacf786beb6dceb630be6af267f242625 (PLAN-004 plus
PLAN-008), SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467, and unchanged G09 CC0
resources from ag-examples d1b4f9baf6f83575af4c669efa9b17f507d9003d. Source
identity covers1431 files, SDK681 files/17 local packages and479 dependency
packages/29 local path edges. It does not consume later005/006 engine changes.

Both candidates use the original three ground segments, upper platform,
three coin coordinates, patrol interval, goal, camera rectangle and controller
(dt1/60, speed4, gravity20, jump8, clearance0.002). Auto owns all gameplay;
Rust supplies fixed input and observes the actual SourceSnapshot/Session/Host
prepare→accept→publish/GPU path. The harness is a separate non-member research
workspace with its own lock/target/consumer preparation, not a root engine change.

| Denominator | Complete-program result |
|---|---|
| G2-01 left/right/release | not_established_budget_fault |
| G2-02 grounded jump/air denial/gravity | not_established_budget_fault |
| G2-03 platform/wall/ceiling/edge/pit | not_established_budget_fault |
| G2-04 three distinct once-only collectibles | not_established_budget_fault |
| G2-05 patrol/contact failure | not_established_budget_fault |
| G2-06 complete collection/goal/Enter new Run | not_established_budget_fault |
| G2-07 following/clamped camera in complete play | not_established_budget_fault |
| G2-08 complete movement/sprite/direction combinations | not_established_budget_fault |
| G2-09 sound and HUD | missing_public_capability |

A writes the current sprite every callback. B only omits an unchanged sprite
setter. Both original-route programs fault with QueryBudget in callback1 and
publish no tick beyond0; the whole last-good Frame and GPU image remain intact.
Consequently original route reachability and later complete-game states are
unproved. Required states are retained as budget-blocked, never counted as pass.

## Calibrated public callback budget

Normal `ag_char_move` costs26:1 precharge +24 v5 move +1 final-support scan.
An initial-overlap/early return can skip the last scan and cost25. These cases
cannot be substituted for the complete support path. The fixed pool stays32.

| Callback source path | Credits / outcome |
|---|---|
| Normal A / B first changed sprite | input3 + move26 + grounded1 + sprite1 + patrol1 + absent-next1 =33; rejected |
| B grounded unchanged idle | same, sprite omitted =32; accepted |
| Grounded character-only boundary control | base31 +1 held getter =32 accepted; +2 getters =33 rejected |
| Full triple+goal first callback N | early-return25 + input3 + grounded1 + sprite1 + patrol1 + absent-next1 =32 accepted |
| Full triple+goal N+1 | event batch is now available; first demand beyond32 refused, hypothetical suffix not established |
| Isolated single N / N+1 / no-duplicate |7 /11 /7 |
| Isolated triple+goal N+1 |22 (three real despawns, four event records plus final absent read) |
| Isolated triple+enemy+goal |25; enemy failure takes precedence |
| Ended no-Enter / Running Enter request / fresh app_start |1 /2 /11 |
| Paused Host frame, Stop or final-owner drop | no Auto callback;0 |

The replayable ledger binds source/case/Run/published tick/attempted tick,
public call counts/prices and early/conditional ranges to actual observations.
It is source-derived and threshold-calibrated, not a private meter reading.
N geometry is read in N+1; callback1's queue is empty. Fault records keep the
old publication and identify refused demand33 and the hypothetical suffix,
not newly published diagnostics. Relative trajectories retain their Run and
absolute predecessor tick. Independent controls reject accepted47-credit,
first-tick getter, shifted Run/tick and ended-state gameplay accounts.

## Isolated evidence, lifecycle and intent boundary

Labelled isolates exercise real motion/release/jump/air-denial/fall, collisions,
patrol endpoints and a frozen-enemy mutant; N+1 single/triple real destruction,
dropped-third-event/restoration, unmet goal, enemy/pit failure, restart/stale
handles and danger priority. Correct/faulty sprite output is distinguished by
actual GPU samples and whole-frame last-good. These isolates supplement the
negative diagnosis and do not turn the complete denominator green.

Twenty short Run/Stop cycles check real VM weak owners and editor restoration
after the next Host draw. Editor/external image pins are valid retention;
last-owner release is separately checked. Fresh no-redecode/owner samples add
to unchanged PLAN-004 lower-level warm texture no-reupload evidence. Base scene
uses11/128 nodes,4/8 live slots,5/8 pairs and one735-byte PNG (20480 decoded and
20480 expected image GPU bytes); diagnostic isolates add hidden leaves only.

Accepted diagnostic isolates expose existing Auto jump/coin/HUD refresh intent
counters through public zero-credit setters on far-away hidden leaves. Jump is
once and an air re-press does not add an intent; single collection remains1,
triple collection yields3 coin intents and4 HUD refresh requests (three coins
plus goal), ended callbacks preserve totals and Enter resets them. Actual
source mutants producing6/0 coin intents are detected before restoration to3.
These are finite event-driven requests, not sound playback or rendered UI.
Original Fault suffix intent values remain unestablished.

## Timing, limits and follow-up

Three release windows each include two original failing callbacks and60 legal
isolated character ticks: maxima2.587/2.795/2.711ms, means about2.06ms. This is
`met_for_measured_prefix_only`; no successful full-route average/max exists.
The debug comparison maximum18.119ms misses8ms. Whole Host tick includes
preparation/draw submission; readback/assertions and nested VM/admission/prepare
windows are separate, not added twice. GPU timestamps are unavailable and
background desktop load is uncontrolled; no production performance claim follows.

All hard limits remain unchanged:32 credits/callback,1000 steps/slice,
100000 cumulative steps/callback,120 resumes/250ms,30s initialization/2s tick,
256KiB user/96KiB managed/352KiB combined source and existing resource limits.
The fixed public Play exposes no exact per-callback step/resume consumption.
The user explicitly accepted recording these fields as unmeasured for r2 with
the SDK limits and other evidence retained. Enforced bounds are not measured
maxima or proof of headroom. No private meter/engine patch was introduced.

Independent consumer replay and report controls retain the full denominator,
source/Git/dependency binding and raw-derived candidate/CPU/classification
consistency. `full_m2_proven=false`. Public sound/HUD, equivalent lower-cost
operations or budget-policy changes need separate approved engine Plans; a
future game belongs in its own repository/Plan and must rerun the complete
original route and peaks after capability changes.

Sources: [research and replay commands](../../../../auto-ui/research/autographics-g2-budget/README.md),
[delivered Plan and review evidence](../../../../auto-ui/docs/plans/archived/011-autographics-g2-full-loop-budget-feasibility.md).
