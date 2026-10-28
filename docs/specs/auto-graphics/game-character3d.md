# Public three-dimensional character (PLAN-874)

Status: experimental, delivered on `autographics-dev` only. This slice extends
the public game-input and character surface with one explicitly selected 3D
character profile; it does not close the full G3 game, a runtime camera, dynamic
instances/triggers, game rules or general physics.

## Public profile `autographics.character3d/1`

An explicit experimental behaviour profile of the Auto source driver, isolated
from `autographics.character/1` (which stays a four-action 2D surface). It does
not occupy `autographics.behavior/6` (that version string stays unclaimed and
rejected); its assembly version marker is 102 and its play profile is 13.
Selection is explicit (panel `API character3d/1`, command
`behavior-api-character3d`, or CLI `--behavior-api character3d`); the legacy
profiles 1..5 and `character/1` keep their byte-identical assemblies, admission
and credit semantics.

## Public surface

| Function | Pricing (shared 32/callback pool) | Semantics |
|---|---|---|
| `ag_char3_bind(stable_id)` | 0 (trusted bookkeeping, no scans) | `app_start` only; binds one **3D** leaf as the run's character; shape pinned to the AABB half extents 0.4 / 0.5 / 0.4 (box 0.8 x 1.0 x 0.8) |
| `ag_char3_solid(stable_id)` | 0 | `app_start` only; declares a static 3D solid (leaf, not the character, deduplicated, <=128, empty allowed) |
| `ag_char3_move(dx,dy,dz,clearance)` | 26 (24 v5 move + 1 reservation + 1 fused final-support determination) | `app_update` only (post-freeze); one call per logical step by construction (2x26 exceeds the pool) |
| `ag_char3_grounded(token)` / `ag_char3_support(token)` | 1 | token 0 = freeze-time neutral support; otherwise the last move's geometric final support; stale -> fault 13 |
| `ag_action_held(action)` / `ag_action_pressed(action)` | 1 | actions 0..5 = Left/Right/Forward/Back/Jump/Restart; pressed consumes once per dispatch tick |

Gravity, speed, jump velocity, dt, the horizontal basis and the gameplay rules
live in the user source; the profile provides only the bounded three-axis
contact operation, the final-support determination and the action snapshot.

## Actions and the single input router

Real OS keyboard events reach the user script through one router: during a
character3d run the mapped keys (A/D/W/S, the four arrows, Space, Enter) feed
only this profile's sampler — the legacy x/y/toggle sampler and the 2D
character sampler never see them, and one key event can never drive two paths.
OS auto-repeat never creates an edge; edges accumulate only while Running;
Pause, Stop, Fault, focus loss, leave, resize/DPI, modality and a new Run clear
held state and pending edges, and a resume needs a fresh press. A neutral Step
dispatches an all-neutral snapshot. Text focus, pointer and scroll never
produce game actions.

## Configuration freeze and read-only enforcement

Configuration happens through the binding calls inside `app_start` and is frozen
when it completes: the character's and every solid's world translation is
snapshotted on all three axes, and a neutral downward support measurement fixes
the initial grounded state (never defaulted). A node declared both solid and
character (solid first, then bind) is rejected at freeze. At each tick begin the
unit verifies all three axes of every solid and of the character against what
the controller last published; external `ag_set_local` / `ag_move_and_slide`
moves are rejected at the next tick boundary. Character-domain faults are 20
(solid moved), 21 (character moved), 22 (config after freeze), 23 (unknown or
duplicate identity, including the cross membership), 24 (invalid target or
unsupported shape), 25 (no character at freeze), 26 (move before freeze).
Visibility never changes collision; hiding a solid keeps it solid.

## Support determination and contact

One move is one v5 `ag_move_and_slide` (at most two contacts) plus one fused
vertical final-support determination written inside the character3d unit: a
solid supports the character when its top face lies in `[feet-0.01, feet]` and
the character centre is strictly inside the direction-expanded footprint (the
v5 boundary rule treats axis-boundary equality as leaving; groups, other
dimensions and unconfigured boxes never support; hidden bodies still do). The
highest such face wins, matching the v5 nearest-fraction result. The scan does
not disturb the v5 result slot, so move tokens keep their own contact data.
Status 3 (initial overlap) reports no support and never counts as grounded;
status 4 (precision stop) still determines final support.

## Measured envelope (debug VM, frozen limits unchanged)

The inherited per-callback guard (1000 steps/slice, 100000 cumulative steps,
120 resumptions, 250 ms) is unchanged. Measured with the real Auto VM:

- 128-node scene, wide layout (bodies spread across the sweep axis, nearest
  blocker iterated last): a moving character with a jump stays inside the
  guard; the nearest-blocker clip matches the independent geometric oracle.
- 128-node deep/contact layout (every body overlapping the moving character's
  footprint every tick): the tick is refused by the unchanged cumulative guard;
  the supported deep contact count with the full controller is 64 nodes.
- 2D `character/1` tolerates the same deep geometry at 128 nodes; the extra
  three-axis sweep cost is what moves the 3D boundary.

This envelope is measured, not a limit change, and the refusal path is a
permanent regression (`over_limit_contact_layout_is_refused_by_the_unchanged_guard`).

**Agreed contract (user ruling 2026-10-28).** The node invariant of this profile
is: *128 scene nodes for wide/sparse layouts, 64 nodes when every body overlaps
the moving character's footprint*. The inherited cumulative step guard is not
raised, the 2D `character/1` profile keeps its 128-node behaviour, and a dense
layout beyond the envelope is refused by the guard (no partial frame) rather
than truncated. The reported sampling shapes are `regular` (3 nodes),
`wide128` (128 nodes) and `deep128` (64 nodes).

## Boundaries

One 3D character, static AABB solids, translation-only motion, no slopes, moving
solids, rotation, dynamic scale or general collision solver, no runtime camera
(the horizontal basis is a user-source seam that PLAN-875 will connect to the
camera), no sprites/audio/HUD, no dynamic instances or triggers, and no game
rules beyond the action snapshot (Restart is delivered to user rules only). The
`.agscene` persistent format is unchanged. Samples: `samples/character3d/`
(two layouts proving geometry sensitivity and the speed 4->5 source-edit
proof). Reproduce with `tools/check_character3d.py --controls` and the named
tests in `tests/character3d_api.rs`, `tests/character3d_host.rs`,
`tests/character3d_samples.rs` and `tests/game_input.rs` (single-threaded).
