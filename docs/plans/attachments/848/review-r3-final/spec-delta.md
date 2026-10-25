# PLAN-848 revision 3 proposed normative delta

These are proposed rules for independent review and merge deposit, not a live
Spec/ledger publication. Existing v1/v2 byte identities and policies remain.

| ID | Target | Proposed rule | Acceptance |
|---|---|---|---|
| SD-01 | auto-graphics/segment-query-profile.md (new) | Finite world AABB closed slab; filtered matching leaves; f32 nearest then Run ordinal; XYZ entry-face tie, inside/boundary normal0; finite geometry before safe nearest pruning; executed arithmetic finite. Callback-scoped one-slot token, miss defaults, explicit errors12/13/14;128 nodes and original CPU limits. | AC-02/04/05/07 |
| SD-02 | spatial-query-profile.md; behavior-api.md; behavior-project.md | Explicit behavior/3/profile9 and scene4 Binding.api admission. Cast8, getter1, inherited AABB1 out of32 credits. Raw source may share Arc; compile/API identity remains version-bound. Old v1/v2 public names, reserved sets and hashes unchanged. | AC-03/05/06 |
| SD-03 | scene-play-session.md | Full spatial configuration before callbacks. Result invalidates on cast attempt and callback boundaries. Last-good frame, sticky Fault, closed VM and Stop recovery unchanged. v3 alone derives dirty X/Y/Z/visibility in parent-first passes from the same shared formula, updates the existing arrays and caches one parent per axis pass. Setters mark components; move marks all axes; initialization/shape recovery is full. Only trusted numeric generated regions use compact line layout under unchanged VM instruction accounting. Clean exits avoid redundant derive; arithmetic order and output remain. | AC-04/05/07 |
| SD-04 | auto-source-driver.md; prototype-profile.md; project.md; integration-contract.md | Beam2D and Sight3D are user-owned Auto nearest-query examples; source change/Update/Save/new-root reopen changes behavior with the same binary. AABB proxies are not exact triangle/OBB picking, physics or camera unprojection. Production I2/I3/ACE remain separate. | AC-01/06/08/10 |

Persistent result state is eleven scalar globals (44B logical payload). Four dirty scalars add16B, so result+dirty logical state is15 scalars/60B. Query
locals and the parent cache live only in the current call; no extra
candidate array, per-cast object/list, geometry descriptor or running VM exists.
Only the specialized original derive allocates initial world/effective arrays, then updates dirty components in place. SDK, AGP and all three
locks retain T-01 identities. Existing memory exclusions remain unchanged.

Safe pruning follows a complete matching-box finite validation. A candidate
may stop slab work when an axis rejects it, or its current entry fraction is
already no smaller than the known best (ascending ordinal guarantees tie
priority). It cannot hide a later unavailable/nonfinite matching box. Skipped
arithmetic is not claimed to have executed; any executed nonfinite intermediate
produces ArithmeticNonFinite. Initial internal sentinel2 is never exposed as
a result; legal miss publishes fraction1 and endpoint B.


Measured on the fixed SDK: slot value storage88B, String structures264B,
owned key capacity138B; logical public state remains44B. Shared-map capacity,
map structure and heap-object count are separate observations with allocator,
compiled-program/stack and driver exclusions, not a claimed total hard quota.


The final winner alone materializes point and XYZ-first entry normal. Bounds and
quotients are explicitly float before finite comparisons. No epsilon, reciprocal
substitution, t-order change or relaxed numeric error policy is introduced.
The original r2 counterexamples and positive/negative/unequal three-axis controls
are part of the permanent128-node/32credit acceptance matrix.


The four dirty flags measure32B NanoValue storage,96B String structures and56B
owned key capacity on the pinned SDK; the11 result values remain88B/264B/138B.
Shared global-map capacity/structure and allocator/program/stack are separate,
not falsely presented as a portable hard total. No extra heap objects are added.
