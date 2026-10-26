# Finite swept AABB profile

Explicit `autographics.behavior/4` selects SweepV4/profile10. Actual Auto computes
fixed-orientation world-axis AABB translation against targets stationary at the
query instant. Geometry is the existing actual all-mesh Descriptor, including
static scale/yaw/pitch and eccentric offsets; this is not OBB/triangle CCD,
relative moving-target prediction, automatic movement, skin, slide or physics.
V1/v2/v3 managed bytes, admitted helpers, entry identity and behavior remain unchanged.

## Query and numerical rules

`ag_sweep_aabb(node, visible_only, dx, dy, dz)` takes a legal same-Run leaf
(node0 valid), exact visible_only0/1 and finite f32 world displacement, not local
motion/velocity. Dimension comes from the node;2D dz is exactly0. A hidden actor
is legal. Candidates exclude self/groups/other dimensions; visible_only1 filters
effective-hidden targets, independent of alpha. Empty candidates give a valid miss.
Setters already accepted in the callback affect the world observed by the query.

For each axis, first compute target world endpoints by the spatial profile.
For static mover offsets[mlo,mhi], form L=tlo-mhi and H=thi-mlo. Intersect the
closed finite slab along O+t*D,t in[0,1], using `(L-O)/D` and `(H-O)/D` in that f32
order. D=+/-0 compares without division. Mover and every matching target world box
must be available/finite before legitimate nearest pruning; later bad geometry
cannot be hidden by a near hit, start overlap or zero displacement. Filtered
leaves do not participate. Bad current world geometry is GeometryUnavailable9;
executed interval/difference/quotient/O+D/result arithmetic nonfinite is
ArithmeticNonFinite12. Check uncoerced world coordinates before typed edge
arithmetic so native NaN through local/return/container paths cannot be misclassified.
Skipped later arithmetic does not create an error.

Initial closed overlap or boundary contact returns hit1,t0,start_overlap1 and
normal0, including displacement away from that contact. Zero displacement otherwise
misses. Noninitial entry takes earliest actual f32 t, endpoint t1 included without
epsilon. Exact ties use lowest Run ordinal; simultaneous entering faces choose
X/Y/Z. Positive entry underflow to0 retains its entering face/start_overlap0.
Returned origin=O+t*D is the mover's world origin, not surface point/box center.
Because finite f32 expressions have specified order, extreme rounding need not
match the preexisting differently ordered discrete overlaps bit for bit.

Hit/miss/node/t ordering, overlap and axis/sign normals use exact oracles. Float
observations use max(1e-6,8*f32EPSILON) for t and
max(1e-3,8*f32EPSILON*max(1,abs(expected),abs(actual))) for world positions;
these tolerances do not expand collision or tie decisions.

## Result identity, budget and lifecycle

Getters: `ag_sweep_hit(token)`, `ag_sweep_node(token)`, `ag_sweep_t(token)`,
`ag_sweep_origin(token,axis)`, `ag_sweep_normal(token,axis)` and
`ag_sweep_start_overlap(token)`. Axis0/1/2;2D z outputs0. Miss is hit0,node-1,t1,
origin O+D,normal0,overlap0. Both hit and miss return a positive token; error0
is a placeholder under a sticky fault, never a successful query.

V4 cast/sweep share one latest typed snapshot and one monotonically increasing
positive i32 sequence for a Run. Every attempt invalidates the old token before
charge/check, even on error; getters reject wrong kind/stale/missing/cross-callback
tokens as InvalidQueryResult13. Exhaustion14 never wraps. Callback begin/end and
Stop invalidate; AABB does not replace the slot. Later setter changes do not alter
a retained result. Tokens are neither persistent IDs nor security capabilities.

Original32 callback credits remain: sweep16,cast8,each result getter1,each original
AABB attempt1. Charge precedes geometry/result access; helpers do not double-charge.
Complete spatial configuration precedes app_start. First error/code/node/argument
sticks; the whole frame is rejected, VM closes and last-good remains. Starting
cancellation, Pause/neutral Step/Resume, Stop editing recovery and fresh Run remain.

18 retained query/count/kind/sequence/dirty scalars are72B logical (limit128B),
with no growing result list, copied scene or private cross-callback cache.
Pinned-SDK observations separately report144B NanoValues,432B String structures,
242B key capacity; shared global map capacity/structure, allocator/program/stack
and fixed containers are separate and are not a portable total128B SDK quota.
Real last-owner release and legitimate source/asset/GPU pins are verified alongside
repeat-query object counts, warm counters AND actual canvas content.

## Supported controls and user policy

2D/3D wide/deep128-node controls, multiple parent transitions and two group writes
around query preserve direct callback-final world publication. One sweep+eight
result getters+eight original AABB attempts is32credits; a smaller cast8+two
getters+sweep16+six getters control also fits32. Original100000steps/120resumes/
250ms callback,30s initialization/2s whole tick/8ms pump limits remain. Observed
yields/wall times are not exact instructions or arbitrary-script performance guarantees.
Descriptor remains Run-scoped, built once from actual resources; no per-tick
vertex scan, source read, model upload, new target/pipeline or readback is required.

Dash2D crosses a thin wall within its requested displacement while its discrete
endpoint would miss; the finite-volume query stops earlier than origin cast.
Dash3D moves an eccentric multipart GLB under a translated parent and fixed
rotation/scale, with multiple targets and same-callback parent/query/child writes.
Actual user Auto selects travel and explicit clearance. Finite tested gaps permit
reverse input; initial overlap/contact gives visible blocked zero movement, not
penetration resolution. Input feedback persists after release for direct node/t/
normal/origin checks. Source Update/Save/SaveAs/new-root new-process reopen changes
speed/clearance with the same runner; OS input and every mesh's final pixels form
acceptance, including partial/transparent/missing-part/wrong-asset rejection controls.
Production AutoUI/RQHost I2, Forge/distribution I3, Schema/ACE/HMR, rotation sweep,
sliding, relative dynamics and a full character controller remain separate work.

Provenance: [PLAN-849](../../plans/archive/849-autographics-swept-aabb-and-stop-before-impact.md), [independent final review](../../plans/attachments/849/review-r1-final/review.md), [frozen merge receipt](../../plans/attachments/849/merge-r1/receipt.md). Graphics source: autographics-dev; production I2/I3 remain pending.

## Bounded-motion extension (PLAN-851)

Explicit v5/profile11 inherits public swept-query geometry and closed initial-contact semantics. Its cast/sweep/motion results share one typed latest slot and positive Run sequence; sweep16/cast8/motion24/getter1/AABB1 share32 credits. V4 identities and behavior remain unchanged. The separate [bounded-motion profile](bounded-motion-profile.md) defines movement response, directional initial-boundary exception and final actual-local admission; these do not change the public sweep query.

Provenance: [PLAN-851](../../plans/archive/851-autographics-bounded-move-and-slide.md), [independent final review](../../plans/attachments/851/review-r4/review.md), [frozen merge receipt](../../plans/attachments/851/merge-r4/receipt.md). Source stays on autographics-dev; production I2/I3 remain pending.
