# AutoGraphics game actions and single 2D character (PLAN-861)

## Public profile `autographics.character/1`

An explicit experimental behavior profile of the Auto source driver. It does
not occupy `autographics.behavior/6` (that version string stays unclaimed and
rejected); its assembly version marker is 101 and its play profile is 12.
Selection is explicit (editor API selector or `--behavior-api character`); the
legacy profiles 1..5 keep their byte-identical assemblies and admission.

## Public surface

| Function | Pricing (shared 32/callback pool) | Semantics |
|---|---|---|
| `ag_char_bind(stable_id)` | 0 (trusted bookkeeping, no scans) | app_start only; binds one 2D leaf as the run's character; shape pinned to the 858 half extents 0.4/0.5 |
| `ag_char_solid(stable_id)` | 0 | app_start only; declares a static solid (2D leaf, deduplicated, <=128) |
| `ag_char_move(dx,dy,clearance)` | 26 normal complete path (1 precharge + 24 v5 move + 1 final-support scan);25 for an initial-overlap/early return that skips final support | app_update only (post-freeze); one complete call per logical step by construction (2x26 exceeds the pool) |
| `ag_char_grounded(token)` / `ag_char_support(token)` | 1 | token 0 = freeze-time neutral support determination; otherwise the last move's geometric final support; stale -> fault 13 |
| `ag_action_held(action)` / `ag_action_pressed(action)` | 1 | actions 0..3 = MoveLeft/MoveRight/Jump/Restart; pressed consumes once per dispatch tick |

Gravity, speed, jump velocity and dt live in the user source; the profile
provides only the bounded contact/support operation and the action snapshot.

## Actions and the single input router

Real OS keyboard events (A/D, arrows, Space, Enter) reach the user script
through one router: during a character run the mapped keys feed the GameInput
sampler only — the legacy x/y/toggle sampler never sees them, and the same
key event can never drive both paths. OS auto-repeat never creates an edge;
edges accumulate only while the run is Running; Pause, Stop, Fault, focus
loss, resize/DPI and a new Run clear held state and pending edges, and a
resume needs a fresh press. A neutral Step dispatches an all-neutral
snapshot. Text focus, pointer and scroll never produce game actions.

## Configuration freeze and read-only enforcement

Configuration happens through the binding calls inside `app_start` and is
frozen when it completes: the character and every solid's world translation
is snapshotted, and a neutral support scan measures the initial grounded
state (never defaulted). At each tick begin the unit verifies solids equal
their frozen translation and the character equals what the controller last
published — external moves through `ag_set_local` or `ag_move_and_slide`
are rejected with fault 20/21 at the next tick boundary. Character-domain
faults are 20 (solid moved), 21 (character moved), 22 (config after freeze),
23 (unknown/duplicate id), 24 (invalid target or unsupported shape), 25
(no character at freeze), 26 (move before freeze). Visibility never changes
collision; hiding a solid keeps it solid.

## Measured bounds (debug profile, five fresh processes)

Tick CPU P50 median 1.075 ms / P95 median 1.902 ms against the 250 ms
callback limit; credits <= 31 of 32; the 128-node load keeps
nearest-blocker-wins with the blocker iterated last and clips at
face - halfwidth - clearance. Host publication equals an independent Session
trace at checkpoints with per-step tick identity; jump apex follows
v^2/2g; Stop restores the editing view; twenty run/stop cycles release VM
and spatial owners; a fault keeps the last-good frame.

## Boundaries

One 2D character, static AABB solids, no slopes/moving bodies/rotation, no
runtime camera, sprites/audio/HUD, dynamic objects, or game rules beyond the
action snapshot (Restart is delivered to user rules only). The `.agscene`
persistent format is unchanged. Sample project:
`samples/character2d/` (two layouts proving geometry sensitivity and the
speed 4->5 source-edit proof). Reproduce with
`tools/check_character2d.py --controls` and the named tests in
`tests/character2d_*.rs` / `tests/game_input.rs` (single-threaded).


PLAN-011 (auto-ui) corrects the historical25-credit table using the fixed004+008
source and fresh32-pass/33-reject controls. This corrects documentation only;
no price or budget was changed. See [complete G2 budget feasibility](g2-full-loop-budget-feasibility.md)
for the distinction between normal/early-return paths and the bounded no-go result.
