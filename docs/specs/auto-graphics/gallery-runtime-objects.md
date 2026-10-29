# G11 Gallery — runtime objects, finite triggers and audited rounds

Status: **delivered** content knowledge (ag-examples/PLAN-001, consumer of the
auto-ui/PLAN-002 engine delivery `0ee5ad0747ec3bb67e1288401fcc31daf3901c6e`).
This document records the enduring rules the G11 gallery examples rely on; it
does not restate the engine implementation.

## Scope

Capability C11: a user Auto program owns a finite per-Run instance table,
explicit trigger pairs over real geometry, one-shot event reads and a
user-requested new round. The gallery ships two dimensions of the same
mechanism: `g11/2d` (public `autographics.triggers2d/1`) and `g11/3d`
(`autographics.triggers3d/1`). Engine: auto-ui `api/runtime_triggers_v1.at`,
`api/runtime_objects_v1.at`, `src/authoring/runtime_round.rs`.

## Object identity and lifecycle

- Instances live in at most 8 slots per Run; identity is
  `run*524288 + slot*65536 + generation`, with generation starting at 1 and
  never wrapping. A stale or foreign-Run handle is a named sticky error; only
  `ag_object_alive` may answer absence with 0.
- Spawn copies an accepted scene template (same dimension, non-group leaf,
  supported kind); instances are non-solid and never enter the document,
  solids, character sweep or camera targets.
- `ag_object_visible` changes presentation only: identity, geometry and slot
  stay, and **trigger eligibility never reads visibility**. Destruction
  (`ag_object_despawn`) is the only operation that revokes eligibility; it
  emits no synthetic exit, and a same-slot respawn is a new generation that is
  never auto-reattached — rebinding is an explicit operation that resets the
  previous state and reports a fresh enter.

## Finite triggers and events

- At most 8 explicit pairs. A pair binds one same-dimension static leaf subject
  (the Run-bound character is legal) to a source that is either another static
  leaf or a live instance bound to its full Run/slot/generation identity.
- Two shapes: shape 0 is a closed-interval AABB overlap on the active axes;
  shape 1 is a 2D circle / 3D sphere whose explicitly configured radius
  (never a GLB bound) hits when the closest point of the subject's actual AABB
  lies within `radius²`. Tangency hits; an AABB corner that is outside the
  disc/sphere does not. The gallery's default radius is 0.5.
- Evaluation runs exactly once per `app_update`, after that tick's movement and
  derive, reading the same final positions; a second evaluate is rejected.
  `false→true` is enter, `true→true` stay, `true→false` exit; at most one event
  per pair per tick, emitted in pair order. Events are read once
  (`ag_event_next` with an explicit 0 absence), and the user callback of tick N
  reads tick N-1's snapshot; unread events are discarded, never queued.
- After evaluate the geometry is locked for the rest of that callback; spawn,
  move, rebind and character move are named rejections while despawn and
  visibility remain allowed.

## Audited rounds

- The user Auto owns Playing / Won / Failed and the item count. A restart is one
  bounded pending marker per tick (idempotent, rejected at tick 0), consumed by
  the host only after the carrying candidate is fully accepted and only while
  Running.
- A new round keeps the same accepted source bytes and fixed configuration,
  re-runs `app_start`, closes the old VM/object table/event state, and publishes
  fresh instances, character and camera initial values at tick 0. Late results,
  events and inputs from the old Run are rejected.
- A held Enter never chains rounds; a paused round does not process a request.

## Gallery content rules (g11/2d, g11/3d)

- Three distinct collectible instances are counted once on first enter and truly
  destroyed; stays never score again; a same-tick triple touch yields three
  distinct ids (engine emits three enters; an idle callback can read all three).
- A goal wins only after all three items are collected; a hazard and an
  out-of-bounds zone fail the round; Won/Failed accept one fresh Enter to open a
  new round that restores count 0, three new instances, character and camera.
- `main-alt.at` in both dimensions is the parameter contrast: it moves the
  hazard onto the walk line and raises the speed/patrol rate, so the real
  trajectory fails at the hazard instead of winning at the goal.
- 3D movement is camera-relative (`ag_cam3_right_x/z`); orbiting the camera
  rotates the world direction, and the off-path void zone at z=-8 is reached
  that way.

## Limits and evidence

- Frozen budget: 32 spatial credits per callback. A character sweep costs 26,
  so a moving callback fits a single event read; an idle callback reads up to
  three. Content must not raise the budget or drop required behaviour.
- 3D material is fixed to `[0.9, 0.3, 0.2, 1.0]`; 3D item identity is carried by
  template id and scale, not colour. 2D may vary colour and scale.
- Platform: windows-native-x64 only. Acceptance is windowless through
  `ag-examples/tools/verify_g11.py` over `checks/g11/gallery_runtime.rs`
  (public loader → real Auto → Host publish → offscreen GPU), 76 oracles with
  per-source hashes in `previews/g11-runtime-objects/acceptance.json`; a
  single-example ZIP is verified offline A→B with tamper refusal and repair.
