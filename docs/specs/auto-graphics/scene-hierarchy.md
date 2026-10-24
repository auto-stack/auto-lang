# Finite authoring scene hierarchy

Finite authoring forest: shared nonzero u64 ID namespace, total128 nodes, same
2D/3D single group parent or null root, complete references and no cycles. Groups
inherit translation and visibility only; no group TRS/resource/material. Stable
parent-first traversal uses group declaration order then object declaration order
among siblings; original objects array independently preserves2D draw order.
No semantic depth truncation within128 nodes. Both local and derived world
coordinates finite abs<=100000, including hidden nodes;2D group z0. Auto profile4
receives local nodes plus checked parent ordinals, computes per-layer f32 addition
and local AND parent visibility. Rust checks all IDs/kinds/dimensions/parents/counts,
numbers and semantics; renderer/pick/outline consume actual returned values.
Any bad packet/resource leaves accepted content/history/selection and preview intact.

Provenance: [PLAN-837](../../plans/archive/837-autographics-scene-hierarchy-and-group-authoring.md), [final R2](../../plans/attachments/837/review-r2/review.md), [merge receipt](../../plans/attachments/837/merge-r1/receipt.md). Source integration remains autographics-dev; production I2/I3 pending.

## PLAN-838 current static asset extension

Import-and-create is one ordinary accepted history step. Import/cancel generation
and stable selection tokens reject old canceled operations and stale resource
replacement targets even when document revision is unchanged. Registered resource rows
create another instance or replace a selected3D leaf while preserving its ID,
parent and all local properties. A selected3D group receives a new same-dimension
leaf. Same SHA reuses asset/CPU/GPU owners; same name with different bytes gets a
different identity. External changes require another Import, with no hot reload.
Cancellation at stage boundaries drops pending asset/VM owners and leaves accepted
history/selection/dirty unchanged; synchronous parse/compile/native GPU calls are
not preempted. Busy edit/gesture/save/navigation semantics retain their previous
meaning. Document/history/registry/GPU publication is one candidate transaction.
Undo/Redo hold needed payloads, including when original import paths disappear.
Registered unused assets remain declared until New/Open/Close; disk residuals are
separate and are not automatically deleted. OBB/pick/group aggregate bounds and
actual renderer consume the same admitted Auto-selected complete multi-part model.

Provenance: [PLAN-838](../../plans/archive/838-autographics-glb-import-and-portable-scene-assets.md), [independent final R2](../../plans/attachments/838/review-r2/review.md), [merge receipt](../../plans/attachments/838/merge-r1/receipt.md). Source remains autographics-dev; production I2/I3 pending.

## PLAN-839 current scene play extension

Document remains autographics.scene/3. Runtime/actor selection/ticks/input/VM/code
bindings are transient and never serialized. Run/Pause/Step/Fault do not mutate
accepted bytes/hash/stamp/selection/saved hash/dirty/undo/redo or entry/GLB files.
All editing/structure/resource/file commands and actor changes are disabled and
rejected again at Host admission until Stop; property drafts must Apply/Cancel
before Run. Runtime local/world state lives outside Editor and never calls its
begin/commit APIs. After Stop, Save and a new-process reopen contain editable locals,
not final runtime transforms. Existing hierarchy/asset budgets remain in effect,
including hidden potential512 mesh instances and full imported multi-part models.

Accepted Registry and GPU mesh Arcs are pinned/shared, never reparsed/reuploaded in
warm runtime frames. Stop releases the additional runtime Session, while Editor's
accepted preview/assets legitimately retain resources. Actual20-run weak-owner/Arc/
target/pipeline identity/upload accounting and retention/reupload negatives distinguish
this from leaked runtime owners or repeated model upload. Final Editor close releases
source payloads; an explicit external mesh pin lasts until its own last drop.

Provenance: [PLAN-839](../../plans/archive/839-autographics-scene-play-mode-and-auto-runtime.md), [independent final R2](../../plans/attachments/839/review-r2/independent/review.md), [affected independent R3](../../plans/attachments/839/review-r3/independent/review.md), [merge receipt](../../plans/attachments/839/merge-r1/receipt.md). Source remains autographics-dev; production I2/I3 pending.
