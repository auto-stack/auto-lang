# Finite segment query profile

Explicit autographics.behavior/3 selects UserSegmentV3/profile9. This finite
experimental world-space API uses the existing all-mesh AABB descriptor; it is
not triangle/OBB picking, an infinite ray, swept collision/physics or camera
unprojection. V1/v2 source identities, reserved names and budgets are unchanged.

## Query and numerical rules

`ag_cast_segment(space,ignore,visible_only,ax,ay,az,bx,by,bz)` returns a positive
token for hit or miss, zero on a sticky API fault. Integers:space2/3,ignore-1 or
a valid same-dimension leaf (handle0 valid),visible_only0/1. Group/wrong-space/
invalid ignore is rejected. Endpoints are finite f32 world coordinates;2D z0.
Groups have no queryable shape. Visibility filtering uses effective visibility,
not alpha; ignored/other-dimension/filtered leaves are not geometry candidates.

Actual Auto traverses candidates in Run ordinal order and uses closed [0,1]
slab intervals. Nearest actual f32 t wins; exact ties retain lowest ordinal.
Parallel signed-zero components do not divide, endpoints touch, zero-length
segments are point queries and finite degenerate boxes are supported. Inside
or boundary starts return t0/normal0; outside entry faces are signed unit axes,
X/Y/Z first for equal entry t. A positive entry that underflows to0 retains its
entry face. Point=A+t*(B-A), with no epsilon expansion or direction normalization.

Every matching box is finite-checked before nearest pruning. Unavailable or
overflowing matching geometry is GeometryUnavailable9, never a hidden miss.
Axis rejection or a proven non-winning entry may skip subsequent arithmetic;
executed nonfinite direction/quotient/point arithmetic is ArithmeticNonFinite12.
Bounds/quotients are explicitly float. The winner alone materializes point and
normal using the same f32 arithmetic; no reciprocal or changed t-order policy.

## Result lifetime and charging

`ag_cast_hit(token)`, `ag_cast_node(token)`, `ag_cast_t(token)`,
`ag_cast_point(token,axis)` and `ag_cast_normal(token,axis)` read one immutable
query snapshot,axis0/1/2. Miss defaults:hit0,node-1,t1,point B,normal0. Subsequent
motion does not change a token's snapshot; explicitly cast again for new world.
Every cast attempt and callback begin/end invalidate the prior slot. Bad/stale
token is InvalidResult13; bad axis is original InvalidAxis2. Positive i32 IDs never wrap and exhaustion is
ResultIdExhausted14. Tokens are neither persistent IDs nor security credentials.

Each v3 callback has32 weighted credits:cast8,result getter1,inherited AABB1;
each public attempt charges before geometry/result access, with no helper
double-charge. Original v1/v2 retain32 AABB attempts. Sticky first error rejects
the entire callback, closes its VM and preserves last-good; placeholders do not
make business success. Complete spatial initialization must precede app_start.
Stop restores accepted editing content; source repair requires a new Run.

## Derived state and bounded verification

Setters feed the original graph:four dirty scalars for X/Y/Z/visibility,move
marks all axes, initial/incomplete shape derives fully. Parent-first axis passes
reuse the shared local+parent f32 formula and existing arrays in place, with one
local last-parent cache per axis pass. No second graph, candidate array or
per-query heap object is created. Clean callback exits avoid redundant derive.
Only trusted v3 numeric generated regions use compact source lines under the
unchanged Parser/Codegen/VM instruction accounting. User raw bytes and old
managed identities are unchanged; generated-region diagnostics use real layout.

11 result scalars44B +4 dirty scalars16B =15 scalars60B logical state (<=16/64B).
Pinned SDK result values88B/key structures264B/key capacity138B; dirty values32B/
key structures96B/key capacity56B. Shared-map capacity/allocator/program/stack
are separate observations and exclusions, not a portable total hard byte quota.

Supported controls:128 wide/deep nodes, two scalar group writes across each valid axis and2casts+10getters+
6AABB=32credits, including original late-axis miss/ordinal-nearer and signed/
unequal three-axis directions. Original100000steps/120resumes/250ms callback,
30s initialization/2s whole-tick/8ms pump/one inflight/four-credit limits remain.
Scheduler yields are not exact instruction counts or a universal headroom claim.
Independent f32 oracle comparisons use max(1e-6,8*f32EPSILON) for t and
max(1e-3,8*f32EPSILON*max(1,abs(expected),abs(actual))) for points; exact hit/miss,
ordering and0/+1/-1 normal rules are not relaxed by observation tolerances.

Beam2D/Sight3D are real Auto nearest-query projects. Same-binary source Update/
Save/new-root reopen changes their logic; Sight3D uses all GLB parts under a
translated parent and static rotation/scale. Native verification uses actual OS
input and all-part top pixels, positive/negative/source/relocation controls.
Legitimate clock auto-Pause is observed and resumed through OS/Iced before
evaluating accepted input; accepted V batches cannot be replayed. Runtime pause
policy, action counts, pixel assertions and original phase deadlines remain.
Production I2/I3/ACE integration and arbitrary-script heap quotas are separate.

Provenance: [PLAN-848](../../plans/archive/848-autographics-segment-casts-and-nearest-hit-interactions.md), [independent affected final review](../../plans/attachments/848/review-r4-final/review.md), [frozen merge receipt](../../plans/attachments/848/merge-r2/receipt.md). Source: autographics-dev; production I2/I3 remain pending.

## Swept-query extension (PLAN-849)

Explicit v4 inherits this unchanged v3 segment geometry but shares its latest cast/sweep result slot and positive i32 sequence with [swept AABB](swept-aabb-profile.md). V4 getters check result kind and use InvalidQueryResult13; v3 retains its original cast-only identity, fees and InvalidResult13 name. V4 sweep16/cast8/getter1/AABB1 share32credits; AABB never overwrites a token.

Provenance: [PLAN-849](../../plans/archive/849-autographics-swept-aabb-and-stop-before-impact.md), [independent final review](../../plans/attachments/849/review-r1-final/review.md), [frozen merge receipt](../../plans/attachments/849/merge-r1/receipt.md). Graphics source: autographics-dev; production I2/I3 remain pending.

## Bounded-motion extension (PLAN-851)

Explicit v5/profile11 retains public segment geometry and adds motion to the same typed latest-result slot used by cast/sweep. Motion24/cast8/sweep16/getter1/AABB1 share32 credits, attempts invalidate before charging, and getters check kind. V3/v4 identities and semantics stay unchanged; see [bounded motion](bounded-motion-profile.md).

Provenance: [PLAN-851](../../plans/archive/851-autographics-bounded-move-and-slide.md), [independent final review](../../plans/attachments/851/review-r4/review.md), [frozen merge receipt](../../plans/attachments/851/merge-r4/receipt.md). Source stays on autographics-dev; production I2/I3 remain pending.
