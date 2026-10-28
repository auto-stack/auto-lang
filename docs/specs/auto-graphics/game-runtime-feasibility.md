# AutoGraphics game runtime feasibility

## M2/S2 first probe: real-Auto character motion within the original hard limits

PLAN-860 ran the frozen G2/G3 one-level samples as real Auto Session probes under
the unchanged budgets (1000 steps/slice, 100k cumulative per callback, 120
resumes/250ms, 32 credits per tick, 256KiB/64KiB/320KiB source, play
30s/2s/8ms) and through the real Host/Play/GPU publication path. The decision
for the follow-up character work is **go**: both compared candidates satisfy
the necessary semantics inside the original limits.

## Frozen mechanics

Two candidates were compared with identical controllers (dt=1/60, speed 4,
gravity 20, jump 8, clearance 0.002, one character-update call per logical
step, frozen route input tables):

- Candidate A composes the public behavior/5 only:
  `grounded := status∈{1,2} ∧ contacts≥1 ∧ dy_req<0 ∧ unapplied_y<0`
  (27 credits/step). The last-normal-only and stale-grounded rules both fail
  the corner/edge gates, so the inference is load-bearing.
- Candidate B is a probe-only trusted unit (`research/game-runtime/character_probe_unit.at`,
  not a public API): the unchanged v5 move plus one fused geometric final-support
  down-scan (skin 0.01) in the same call; grounded is purely geometric
  (26 credits/step, 2 scans/step, 3 extra scalar globals).

Measured facts that hold for both candidates: floor+wall corners stay grounded,
edges drop support, air re-jumps are denied, ceilings classify without
grounding, initial overlap is status 3 without motion or fault, PrecisionStop
reverts are classified (not faults), wall-top landings ground, faults keep
last-good and the sticky bridge skip, credit overflow faults at 32, budget
exhaustion closes the VM without partial frames. G2 and G3 routes reach all
three collectibles and the goal under the frozen tables with clean
enemy/hazard/pit oracles; the 128-wide/deep loads keep nearest-blocker-wins
with the blocker iterated last. Real-mutation controls (stale grounded,
last-normal-only, dropped integration output, frozen input) all go red on the
same gates and restore green.

## Budgets and presentation

Debug-profile tick CPU medians (5 fresh processes per class/candidate):
G2 route 1.5/1.6ms, G3 route 1.9/2.3ms, 128-wide 13.2/15.4ms,
128-deep 15.3/17.5ms (A/B), all ≥100× inside the 250ms callback limit;
working set ≤67MB; observed credits ≤27 of 32. G2 and G3 trajectories
published through the real Host/Play/GPU path match the Session trace bitwise
at init/jump/land/final checkpoints; Stop restores editing pixels exactly; 20
run/stop cycles release VM and spatial owners (weak references); a fresh Run
starts at the frozen spawn. G3 pixels at the default 3D editing camera do not
visibly change across checkpoints — 3D publication is bound numerically plus
restore-exactness, and runtime camera framing stays follow-up work.

## Boundaries

This delivers feasibility evidence only: no public character/action API, no
user admission, no OS input, no sprites/audio/HUD/camera, no game loop. The
follow-up (PLAN-861) freezes the public actions and single 2D character
profile over this evidence and must decide the public pricing of B's fused
support (candidate A remains the fallback composition).

Provenance: [PLAN-860](../../plans/860-autographics-game-runtime-feasibility.md),
[probe contract](../../plans/attachments/860/work/probe-contract.md) (r4),
[decision](../../../experimental/autographics-auto-driver/research/game-runtime/decision.json).
Source: autographics-dev at the PLAN-860 landing commit; SDK
`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`.
