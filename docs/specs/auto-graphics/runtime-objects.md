# Runtime objects: finite run-instance identity and lifecycle

Status: experimental delivered (auto-ui PLAN-001); independent final review pass on delivery `537122b` (auto-ui master, linear history `3f39f5b` + `537122b`). Sources live in the auto-ui repository (`experimental/autographics-auto-driver/`); this document records the enduring public contract. The gallery/G11 consumers stay in ag-examples and wait for the trigger plan (auto-ui PLAN-002).

## Profiles and assembly

Two combined public profiles join the admission table beside the unchanged
`character/1`, `character3d/1` and both camera profiles (their assemblies,
outputs and selectors stay byte-identical):

- `autographics.objects2d/1` — API number 105, wire profile 16, assembled from
  the unchanged v5 motion chain + character2d + camera2d + the objects unit.
- `autographics.objects3d/1` — API number 106, wire profile 17, the same chain
  with character3d + camera3d.

One trusted unit (`api/runtime_objects_v1.at`) serves both dimensions; the
assembly freezes the Run dimension by replacing the declared literal
`var _ag_obj_dim=0` (require-guarded), so no runtime call can flip it. The
character and camera freezes stay after `app_start`; the object table freeze
`_ag_obj_freeze()` runs BEFORE `app_start` (the table must exist for the first
spawn). Every callback entry chain runs `_ag_char*_tick_begin();
_ag_obj_tick_begin(); app_update(dt)` — the latter only resets the per-callback
lifecycle write counter. CLI/panel selection: `--behavior-api objects2d|objects3d`.

## Identity

The public handle is one integer: `run*524288 + slot*65536 + generation`,
slot in `[0,8)`, generation in `[1,65535]`, all integer arithmetic (no float
truncation, no low-bit aliasing; the encoded maximum is exactly `i32::MAX`).
Generation increments monotonically per slot across despawn/respawn cycles;
a rebirth must exceed the per-slot high-water. Exhaustion is named and never
wraps: generation above 65535 and run-id above 4095 both fault code 36. A
handle from an older Run decodes below the current Run base and is dead
(`ag_object_alive` answers 0 — the only legal absence check; every other
consumer of a stale/foreign/old-generation handle is a sticky first error 33).

## Public surface (all calls cost 1 credit of the unchanged shared 32/callback pool)

`ag_object_spawn(slot, template, x, y, z) -> handle` (empty slot, same-dimension
non-group leaf of the accepted base document, supported kind, finite coords,
2D z=0; 5 parameters remain a user-side call — the host Session profile caps
direct calls at four), `ag_object_despawn(token)`, `ag_object_move(token, x,
y, z)`, `ag_object_visible(token, flag)` (visibility only — never a despawn),
`ag_object_alive(token) -> 0|1`, `ag_object_x/y/z(token)`. Lifecycle writes
(spawn/despawn/move/visible) are capped at 32 per callback, checked BEFORE the
credit charge (the 33rd write faults 37, not the budget 10). Sticky error
codes 31–38: slot domain, occupied slot, not-alive, invalid template, domain
(non-finite/out-of-range/2D z/bad flag), identity exhaustion, write cap,
configuration. `0.0/0.0`-style NaN sources fail closed at the VM arithmetic
boundary before any spawn.

## Admitted block and transition admission

Every objects-Run tick appends a fixed block after the camera block: 4 header
ints (status, count, run, tick) plus 8 canonical rows of
(generation, template run index, x, y, z, visible); free rows are all zeros.
Play admission validates the block length (`4 + 22N + camera + 52`), every row
domain, template eligibility against the frozen base order, and the legitimate
birth/death/reuse sequence against the previous admitted frame plus the
host-side per-slot generation high-water (cloned locally; a rejected candidate
can never poison it). Objects, character and camera are one transaction:
prepare/accept/publish together, failure keeps the last-good frame+instances
pair, Stop closes the VM owner and a fresh Run starts with an empty table.

## Host rendering and ownership

Admitted instances render through a non-persistent adapter object (`id = 0`,
which no document object can carry): frozen template kind/scale/yaw/pitch/
material/resource, instance position/visibility, 2D projected through the
active camera/view, 3D as MeshInstances whose hidden-template meshes upload
once on first birth and are shared afterwards. Every live slot pays its full
template mesh count in the potential-mesh denominator (hidden or not). The
construction gates reject named: `objects slot capacity` (base nodes + 8
reserved slots <= 128) and `objects mesh envelope` (base 3D meshes + 8 x worst
eligible template meshes <= 512); prepare re-checks the live denominator per
frame. Instances never enter the Document, scene4, dirty tracking or Undo/Redo,
are never solid, and are never character/solid/camera targets. Mesh/model
resources stay shared across births: the 20-cycle churn gate shows flat upload
deltas with the VM owner closed on every Stop.

## Envelope and evidence

The frozen support set is one dimension per Run and 8 reserved slots over the
inherited 128-node denominator (120 base nodes + 8 slots is the accepted
edge; 121 base nodes is rejected). Sampling shapes `objects2d/3d-regular`,
`objects2d-wide120` and `objects3d-dense64` run five fresh processes each with
8 live instances; every warm-tick p95 stays far below the 250ms whole-tick
line. Evidence: session suite `tests/runtime_objects_api.rs` (V-01/V-02:
identity, admission counterexamples, exhaustion, caps, the reserved
three-item-collection combination), host suite `tests/runtime_objects_host.rs`
(V-03: same-tick co-publication, last-good, churn/owner, envelopes, multi-mesh
GLB template), samples suite `tests/runtime_objects_samples.rs` (V-04: both
dimensions, parameter-twin sensitivity, template-swap mutation), current
source identity `research/plan001-identity.json` verified by
`tools/verify_plan001.py --controls` (the PLAN-879 migration identity remains
as the historical mapping gated behind it), fresh own-SDK consumer offline
locked build, and `tools/check_runtime_objects.py` over the env-gated churn/
sampling reports.

Provenance: auto-ui PLAN-001 (r4), archived at
`auto-ui/docs/plans/archived/001-autographics-runtime-object-lifecycle.md`;
delivery `537122b09c025191355e260bc8e9880a4358cda2` on auto-ui master (linear:
`3f39f5b` work + `537122b` review fix), independent final review pass bound to
that commit. SDK pin `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467` and both AG
locks unchanged; ag-examples and the trigger plan untouched. The trigger and
round-restart seam reserved for auto-ui PLAN-002 is exactly this table's
admitted geometry/generation data.
