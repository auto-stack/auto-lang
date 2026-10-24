# AutoGraphics experimental behavior API v1

`autographics.behavior/1` is a finite trusted local script interface, not a stable
engine SDK or a security sandbox. Use the runner's normal entry:

```powershell
autographics-authoring --open C:/project/scene.agscene --behavior main.at
```

Choose **Use user script** and a `.at` path in Editing, then Run for a temporary
override. Without `--behavior`, scene4 uses its accepted bound source/roles, or
Builtin6 when behavior is null. A relative script path resolves against the saved scene entry's
directory; an unsaved scene requires an absolute local path. Save As changes that
root for temporary file resolution. The UI shows the resolved path/API and the last Run's source hash; failure
does not silently select Builtin6. Active Starting/Running/Paused/Faulted modes
reject source/path changes, without queuing them.

Your complete file defines exactly `fn app_start()` and
`fn app_update(dt float)`, returning Unit. `app_start` runs after the complete
accepted graph/resource/neutral-input configuration, exactly once per new VM.
`app_update` runs once per dispatched tick, with fixed f32 `dt=1/60`.
Ordinary private functions, state and control flow execute in the actual Auto VM.
Unit return admission follows all expression operands in each callback. Nested
functions/lambdas/closures retain their own ordinary value-return semantics.
Top-level initializers are finite literals/pure constant expressions, including
references to already initialized const/immutable let bindings and constant
formatted/backtick strings. Mutable or forward/cyclic reads and assignments are
not constant expressions; put calls
and other execution inside app_start. Imports/include, FFI, async and compile-time
execution are outside this single-file profile. There is no user `main`, reset,
tick or profile implementation. Do not distribute the managed transport as user
business code.

## Functions

Here `node`, `index`, `space`, `axis` and `flag` are int, coordinates and `dt`
are f32 Auto float. Setters return Unit. The authoritative declarations are
`experimental/autographics-auto-driver/api/behavior_v1.at` (autographics-dev), compiled with the shared Builtin6 graph units;
the runner privately namespaces trusted transport tokens, leaving user bytes intact.

| API | Result / behavior |
|---|---|
| `ag_node_count()` | int; all groups and leaves |
| `ag_node_at(index)` | int handle, parent-first ordinal |
| `ag_valid(node)` | int 1/0; `ag_valid(-1)` is a legal absence check |
| `ag_actor(space)` | selected 2D/3D actor, space2/3; absent actor−1 |
| `ag_space(node)` | int2/3 |
| `ag_is_group(node)` | int1/0 |
| `ag_parent(node)` | parent handle or−1 for a root |
| `ag_local(node,axis)` | current local float translation |
| `ag_world(node,axis)` | current world float translation |
| `ag_local_visible(node)` | int local visibility |
| `ag_visible(node)` | int effective inherited visibility |
| `ag_set_local(node,axis,value)` | finite translation; final whole-frame validation applies |
| `ag_move_local(node,dx,dy,dz)` | translate local position |
| `ag_set_visible(node,flag)` | local visibility, flag0/1 |
| `ag_input_space()` | dispatch input space0/2/3 |
| `ag_input_x()`, `ag_input_y()` | sampled −1/0/1 axis |
| `ag_toggle()` | sampled V edge parity0/1 |
| `ag_tick()` | current logical tick, initially0 |

Handles are ordinals scoped to one Run; handle0 is valid. They are not persisted
IDs or unforgeable references. Axis0/1/2 means x/y/z; 2D z must stay0. Setters
change only local translation/visibility. Topology, count, identity, resource,
kind, scale, angles and material remain immutable.

Getters observe earlier writes in the same callback, including a parent group's
effect on a child. Auto performs the derivation; Rust checks the returned complete
world before rendering it. Invalid handles/axis/space/flags set a sticky first API
error, with code/node/argument. Safe placeholder getter returns do not turn that
error into success. The whole callback output is rejected, the VM closes, and the
last complete frame remains. Stop restores the original accepted editing content.

## Source identity, errors and limits

Each new Run reads one regular local file through the existing ancestor reparse
checks, with a bounded read, strict UTF-8, no BOM, at most256KiB. Network/device/
linked paths are rejected. API+bridge are at most64KiB, combined source320KiB.
CRLF and LF hash their actual original bytes differently. The snapshot records
user path/bytes/hash, version/API/bridge/combined hashes, source regions and the
accepted scene/resource/actor identity. Changing a file during a Run does not
reload it. Stop and Run again reads a new snapshot, with new globals and VM.
Forged versions, hashes, region mappings or scene identities are rejected.

Actual Parser AST admission checks callbacks, duplicate root names, reserved
transport/private symbols, dependencies and top-level execution. Normal local
value bindings may reuse old transport names; `_ag_` identifiers remain private.
Comments/string contents are not declarations. The SDK dependency scanner fix
also keeps them inert during later compilation and fresh/cached module service
loading; it does not add module support to the user profile.

Read/parse/admission/compile/callback/output/budget errors name the source and
stage. Structured SDK compile spans map to the user or a named managed region.
Runtime/native errors without SDK spans say no mapped line is available. Runtime
diagnostic messages are capped at8KiB with an explicit truncation marker; a path
and stage prefix are separate. The original SDK error text is retained within
that bound. Fix the source, Stop, and start a fresh Run to recover.

Starting cancellation retains the complete initialization boundary and publishes
tick0 directly Paused; only Resume starts dispatch. Pause does not call callbacks.
Step performs one neutral tick. Existing1-inflight/four-credit/8ms shared pump,
100000-step/120-resume/250ms callback,30s initialization and2s whole-tick budgets
apply to user code plus API/derivation/output together. Synchronous compile/native
calls are not hard-preemptible. The128-node supported control is verified.
There is no hard byte quota for arbitrary user private containers or SDK-native
allocations; owner release tests do not imply such a quota. Source metadata is
one bounded snapshot, not appended per tick or stored in undo/history. Timing/
action reports remain capped at4096; one tick0 hash is kept without another frame.

Scene format remains schema3. Geometry Save/Save As neither writes the script nor
persists its path/API/actor selection in Document. Copy a project's scene/assets/
script yourself and pass the script explicitly at its new root. No script package
manifest, automatic publish, hot reload, dynamic resources/physics, production
AutoUI/RQHost, I2/I3 or complete public engine promise is added.



Provenance: [PLAN-840](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md), [affected final review](../../plans/attachments/840/review-r2/review.md), [complete merge receipt](../../plans/attachments/840/merge-r1/receipt.md). Graphics source: autographics-dev. Production I2/I3 remain pending.

## Scene4 project default (PLAN-845)

The public behavior/1 declarations, units, finite source validator, callbacks, limits and trusted-code boundary are unchanged. A scene may now explicitly bind one accepted byte-exact source and nullable stable same-dimension leaf roles; [behavior projects](behavior-project.md) defines persistence and ownership. Default Run snapshots accepted bytes without file reread, with current-root verified blob or explicit memory/hash provenance. Explicit user-file selection still reads once per Run and remains temporary; saving never silently binds it. Bind/Open compile/check but do not execute callbacks.

Provenance: [PLAN-845](../../plans/archive/845-autographics-bound-behavior-and-portable-projects.md), [independent final review](../../plans/attachments/845/review-r8-preparation/review.md), [merge receipt](../../plans/attachments/845/merge-r1/receipt.md). Source: autographics-dev; production I2/I3 pending.
