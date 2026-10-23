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
