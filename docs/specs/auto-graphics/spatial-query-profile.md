# Finite spatial query profile

PLAN-846 delivered finite experimental profile; source on autographics-dev.
Behavior/2 explicitly selects UserSpatialV2/profile8, beside unchanged behavior/1
and profile7. Scene4 structure is unchanged; Binding.api accepts exactly /1, /2, /3 or /4. Unknown APIs fail, old readers reject /2 instead of falling back.

A Run binds accepted node order/stable IDs, dimensions, actual resource hashes,
static scale/yaw/pitch, geometry algorithm aabb-TRyRxS-f32/1 and source/API/bridge/
base identity. Host borrows actual builtin/accepted Registry ModelData; all mesh
positions contribute to each cached local box, once per resource per Run. Eight
corners under Ry*Rx*S give offsets; Auto adds current world translation. Groups
only translate/inherit visibility and have no queryable shape. 2D Circle/Quad
proxy uses ±scale.xy/2, z0. 3D proxies may include empty/concave space and rotation
padding; existing ellipse/OBB picking stays unchanged. Finite degenerate boxes
are legal; unavailable/overflowing geometry is flagged per leaf, not fabricated
as a zero box. Hidden/alpha0 leaves remain queryable.

ag_aabb_min/max(node,axis) return float endpoints; axis0/1/2, 2D z0.
ag_aabb_contains(node,x,y,z) returns int0/1 for a finite world point (2D z0).
ag_aabb_overlaps(a,b) returns int0/1 for same-dimension leaves. Intervals are
closed, self overlap is1, exact contacts hit and ±1ULP separation distinguishes
misses. No epsilon expands boolean tests. Reference endpoint comparisons allow
max(1e-3,8*f32EPSILON*max(1,abs(expected),abs(actual))) only for float observations.
Actual Auto queries see parent/leaf writes in the same callback through dirty
world derivation. Rust prepares static geometry and checks complete returned
frames; it does not compute user hits or trigger business state.

Exactly32 public query attempts per app_start/app_update;33rd fails before
geometry access. Helpers do not double-charge. InvalidHandle, InvalidAxis,
GroupNotQueryable, InvalidParameter, SpaceMismatch, GeometryUnavailable,
QueryBudget and SpatialConfiguration are named sticky first errors. Placeholder
results are not success: the entire callback is rejected, VM closes, last-good
remains. All bounded geometry configuration precedes app_start; partial or
erroneous metadata cannot dispatch it or publish a frame. Starting cancellation,
Pause/Step/Resume/Stop and all original CPU/time/queue budgets remain.

At most128 rows of availability+6 floats (3584B logical raw content), with
bounded extra transport/container/identity overhead. New Run rebuilds against
current accepted resources and static fields; no per-tick rescan, disk read,
model upload, target/pipeline allocation or readback is required. Descriptors
remain Run-scoped, never scene/history/source-blob data. Actual owners and
legitimate external pins determine release. Source examples implement their own
enter/stay/exit comparison in Auto, with app_start only establishing a baseline.
No engine event queue, precise collision/response/rigid body/raycast/index or
production AutoUI/RQHost/I2/I3 promise is added.

Provenance: [PLAN-846](../../plans/archive/846-autographics-spatial-queries-and-trigger-interactions.md), [final affected review](../../plans/attachments/846/review-r6/review.md). SDK/root and graphics delivery identities are recorded in the Plan consolidation receipt; production I2/I3 remain pending.

## Finite segment extension (PLAN-848)

Explicit v3 inherits the same Descriptor/geometry/instant-world AABB functions and adds [finite segment queries](segment-query-profile.md). V3 charges32 weighted credits (cast8/getter1/AABB1); v2 retains32 original AABB attempts. Shape availability and filtered-candidate/finite-geometry ordering are unchanged; no new geometry cache or per-frame rescan/upload is introduced.

Provenance: [PLAN-848](../../plans/archive/848-autographics-segment-casts-and-nearest-hit-interactions.md), [independent affected final review](../../plans/attachments/848/review-r4-final/review.md), [frozen merge receipt](../../plans/attachments/848/merge-r2/receipt.md). Source: autographics-dev; production I2/I3 remain pending.

## Swept-query extension (PLAN-849)

Explicit v4/profile10 inherits these all-mesh static offsets/current-world AABB functions and adds [swept AABB](swept-aabb-profile.md). Scene4 accepts /4 without changing fields or older versions. Visible-only filters targets, not the moving actor, and alpha is independent. No new Descriptor/geometry cache, per-frame scan/upload/readback or local movement is hidden in a query.

Provenance: [PLAN-849](../../plans/archive/849-autographics-swept-aabb-and-stop-before-impact.md), [independent final review](../../plans/attachments/849/review-r1-final/review.md), [frozen merge receipt](../../plans/attachments/849/merge-r1/receipt.md). Graphics source: autographics-dev; production I2/I3 remain pending.

## Bounded-motion extension (PLAN-851)

Explicit v5/profile11 inherits actual all-mesh offsets, current-world and visibility rules. Scene4 accepts /5 without structural change. [Bounded motion](bounded-motion-profile.md) publishes one accepted leaf-local candidate to the same world array; no second graph, per-tick vertex rescan, model upload or readback is introduced. Older profiles retain their original behavior.

Provenance: [PLAN-851](../../plans/archive/851-autographics-bounded-move-and-slide.md), [independent final review](../../plans/attachments/851/review-r4/review.md), [frozen merge receipt](../../plans/attachments/851/merge-r4/receipt.md). Source stays on autographics-dev; production I2/I3 remain pending.
