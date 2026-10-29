# Runtime triggers and audited round restarts

Status: experimental delivered (auto-ui PLAN-002); independent final review
pass on delivery `0ee5ad0` (auto-ui master, linear history `537122b` +
docs + `0ee5ad0`). Sources live in the auto-ui repository
(`experimental/autographics-auto-driver/`); this document records the enduring
public contract on top of [runtime-objects.md](runtime-objects.md), whose
identity model, admitted block and host adapter it consumes unchanged. The
gallery/G11 consumers stay in ag-examples and consume this delivered closure.

## Profiles and assembly

Two combined public profiles extend the admission table beside the unchanged
objects profiles (their assemblies, outputs and selectors stay byte-identical):

- `autographics.triggers2d/1` — API number 107, wire profile 18, assembled
  from the unchanged v5 motion chain + character2d + camera2d + the objects
  unit (dim 2) + the triggers unit.
- `autographics.triggers3d/1` — API number 108, wire profile 19, the same
  chain with character3d + camera3d + the objects unit (dim 3).

One trusted unit (`api/runtime_triggers_v1.at`) serves both dimensions; the
assembly freezes the dimension by replacing the declared literal
`var _ag_trg_dim=0` (require-guarded). Trigger profiles imply the objects
profile: the same 8 reserved slots, slot-capacity and mesh-envelope gates.
Entry chain: `_ag_trg_freeze()` before `app_start` (the pair registry must
exist for the first bind); `_ag_char*_tick_begin(); _ag_obj_tick_begin();
_ag_trg_tick_begin(); app_update(dt)` (the latter resets the per-callback bind
counter, the evaluated latch, the geometry lock and the request marker); and
exactly one `_ag_trg_evaluate()` after the post-callback derive — the unique
`toggled=0}` tail of the tick body, so tick 0 never evaluates. CLI selection:
`--behavior-api triggers2d|triggers3d`. The managed bridge envelope was raised
64→96KiB (combined 320→352KiB) by auto-ui PLAN-002 r5 with the recorded
counterexample (the triggers composition measured 72141/74050 bytes against
the old limit; Objects2V1 fits at 61950) — an explicit, guard-protected raise;
the 256KiB user-source limit and every runtime budget are unchanged.

## Explicit pairs and shapes

At most 8 explicit pairs per Run (stable ordinals = registry order). One pair
binds one same-dimension static leaf subject (the character node allowed) to
one source: either a static leaf or a live PLAN-001 instance bound to its full
Run/slot/generation handle. Two shapes: 0 = closed-interval AABB overlap on
the active axes (touching counts; a 1ULP gap separates), 1 = 2D circle/3D
sphere whose explicitly configured radius (never a GLB bound; ∈ (0,100000],
finite) hits when the closest point of the subject actual AABB sits within
radius² (tangent hits; a corner inside the box but outside the sphere does
not). Positions: subject/static source = derive worlds + descriptor rows;
instance source = template row + live instance translation. Hiding never
cancels a trigger; triggers are never solid; destruction revokes eligibility.

## Evaluation, events and the read cursor

Evaluate runs exactly once per `app_update` after all this-tick movement and
the post-derive world refresh, reads the same final positions and freezes the
snapshot: false→true enter (1), true→true stay (2), true→false exit (3), at
most one event per pair per tick, in pair order, at most 8 per tick. A second
evaluate in one tick is the named lock error 40. Events are read once each:
`ag_event_next()` advances the cursor and answers an explicit 0 on absence
(never a sticky error); the getters (`ag_event_pair/phase/generation/
source_kind/source/geometry`) answer the current token only and expire to 0
after absence or the next evaluate. Events carry the binding generation and
the originally bound source reference (stable leaf id or the full object
handle, directly despawnable). Rebinding is explicit: a new binding
generation per pair (≤65535, named exhaustion 42, monotone on the host
side), the previous contact resets to false, old events never cross a
generation, and a same-slot new generation is never auto-attached.

## Post-evaluate geometry lock

After evaluate the geometry is locked (`_ag_obj_locked`, injected at assembly
into the objects/character unit derivatives — the delivered `api/*.at` bytes
are never modified): `ag_object_move`, `ag_object_spawn` and the character
move fault 40 until the next tick begin, while `ag_object_despawn` and
`ag_object_visible` stay allowed (collection is legal after evaluate). The
host mirrors the once-per-tick rule through the admitted `evaluate_seq`
(block header) equal to the tick number.

## Admitted rounds block and independent admission

Every triggers-Run tick appends a fixed block after the objects block:
8 header ints (status, run, tick, event count, restart flag, restart tick,
evaluate seq, canonical 0), 8 binding rows of (8 ints + radius float) and 8
event rows of 8 ints in pair order — 144 values. Unbound/inactive rows are
canonical zeros. Host admission is NOT count-checking: the block length
(`4 + 22N + camera + 52 + 144`), every domain, the binding generations against
the per-pair watermark (frozen configuration within one generation, mirroring
the objects rule), instance sources against the candidate instance table
(slot/template/generation exact), and a full independent geometry
recomputation from the derive worlds + descriptor rows + candidate instances
whose expected enter/stay/exit sequence must equal the emitted events row by
row. Tick 0 is the registry frame: no events, no restart, evaluate_seq 0.

## Audited round restart

The user Auto owns Playing/Won/Failed and asks with one bounded marker
(`ag_round_request()`: at most one pending per tick, same-tick merges are
idempotent, tick 0 named-rejected, 1 credit). The host consumes the marker
only from a fully accepted candidate while Running — Starting, Paused and
step ticks never process a round. The restart swaps in a fresh Run/VM on the
same scheduling stack from the same accepted source bytes, actors and
configuration: new run id (atomic, exhaustion named, no wrap), `app_start`
re-runs, user globals reset, objects/events/camera restart at their initial
values, the pair registry and binding generations start fresh. The editing
`play_restore` and the last-good presentation stay untouched: until the new
tick 0 is fully prepared/accepted the old complete presentation stays
published; a failed new round (construction or first preparation) keeps that
last-good under the named `round restart:` fault, closes the new VM and never
auto-retries; Stop still recovers the exact editing Prepared. Old-Run
handles are dead in the new Run (foreign-Run decoding, sticky 33); held
edges never double-restart (OS repeat produces no edge, the new VM's edge
buffer starts empty).

## Named errors and pricing

New sticky codes 40 (TriggerLocked: post-evaluate writes/second evaluate),
41 (TriggerBinding: pair/subject/source domain, stale handles, tick-0
restart), 42 (binding-generation exhaustion). The objects 31–38 and the
shared codes 1–15 are unchanged. Measured pricing inside the unchanged
shared 32-credit callback pool: bind = 1 lifecycle op (≤32/callback, code 37)
+ 1 credit, `ag_event_next`/each getter = 1 credit, `ag_round_request` =
1 credit, evaluate = 0 credits (engine-internal, bounded by 8 pairs). One
character sweep is ≈26 credits (1+24+1), so a moving callback carries at
most one bounded event read — the frozen engine fixtures are shaped by this
(bounded read loop, coins spaced two world units apart).

## Envelope and evidence

The frozen support set is the objects envelope (one dimension per Run, 8
reserved slots/pairs over the 128-node denominator, 512-mesh worst-case
envelope) plus the 96KiB managed-source envelope above. Evidence: session
suite `tests/runtime_triggers_api.rs` (V-01/V-02: the independent geometry
oracle with exact-touch and ±ULP counterexamples, once-per-update evaluate
reading final positions, cursor once-only semantics, rebind generation
resets, destruction revocation, the lock with despawn allowed, every named
binding rejection, restart-marker rules, the whole-path budget probes),
host suite `tests/runtime_round_host.rs` (V-03/V-04: the audited swap with
fresh run/generations/camera, the failure path keeping last-good and
recovering on Stop, held-edge and Paused/step gates, twenty restart cycles
closing every old VM owner, corrupted-block mutants faulting at the named
admission stage), samples suite `tests/runtime_trigger_samples.rs` (V-05:
both engine fixtures play the full loop, the parameter twin fails at the
hazard on a divergent trajectory, the template mutation changes admitted
identity and published content), the lib admission unit tests and the
managed-envelope guard, current source identity
`research/plan002-identity.json` (1415 hashes) verified by
`tools/verify_plan002.py --controls` (the plan001 generation remains as the
historical mapping gated behind it), the batch gate
`tools/check_runtime_triggers.py` (identity + the three PLAN-002 suites +
the three PLAN-001 replays + lib gates), and a fresh own-SDK consumer
offline locked build running the session suite green.

Provenance: auto-ui PLAN-002 (r5), archived at
`auto-ui/docs/plans/archived/002-autographics-trigger-events-and-round-restart.md`;
delivery `0ee5ad0747ec3bb67e1288401fcc31daf3901c6e` on auto-ui master
(rebased from work commit `bfcc0c8f5da6d7086d2849828d22ff9123f90631`,
range-diff fully equal), independent final review pass bound to `bfcc0c8`.
SDK pin `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467` and both AG locks
unchanged; ag-examples untouched. Downstream (ag-examples PLAN-001, G11)
consumes this delivered closure and freezes its own inputs from it.
