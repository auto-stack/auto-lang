# Finite authoring viewport interaction (PLAN-836)

Windows x64 single-window finite authoring profile supports circle/quad exact 2D rectangle/ellipse picking (visible alpha>0; last drawn first) and cylinder/player/mob all-mesh-union AABB transformed by rendered TRyRxS as an OBB. Model proxy includes empty space; triangle accuracy is not promised. CPU unprojects the actual Scene3dContent orthographic camera using wgpu NDC0..1 and finite near/far segment; world segment t selects closest proxy, stable ID breaks ties. Invalid/singular coordinates/transforms or clip misses never select. Press resolves through actual CompositionInput Surface/local, against presented geometry and epoch/revision/camera; changed unpresented camera is rejected. Same stable selection drives list, properties, overlay; selection alone does not alter saved state/history. 2D absolute dx/negative dy moves;3px threshold.3D worldX/Y/Z handles54px/7px band, distance then axis order, projected unit length<1px or clipped origin disabled; no camera motion/multiselect/rotation/scale gizmo. Overlay edges use bounded dots (<=240 segments per edge) within existing4096 draw capacity.

Provenance: [PLAN-836](../../plans/archive/836-autographics-viewport-picking-and-translation.md), [final R2](../../plans/attachments/836/review-r2/review.md), [merge receipt](../../plans/attachments/836/merge-r1/receipt.md). Source integration is autographics-dev only; production I2/I3 remain pending.

## PLAN-837 current hierarchy contract

Real stock-Iced2D/3D tree selects stable group/leaf IDs, explicit parent target/root
buttons and Keep world attach; local properties plus read-only actual world/effective.
Leaf geometry picking still selects leaves. Tree-selected group aggregates actual
visible descendant geometry bounds with one outline and origin translation handle.
2D origin cross10px hit radius,3D existing54px/7px X/Y/Z handles. Empty/hidden/alpha0
descendants or clipped/degenerate origin disable group handles with displayed reason;
tree properties remain. All overlay commands within4096. Indentation display capped8
without dropping deep nodes. Translation-only ancestors make target local delta equal
the world pointer delta while preserving parent offset; group drag changes group local
only, leaf drag changes leaf local only. Actual presented snapshot/camera/stamp drives
render/pick/outline.836 capture, bounded samples, last endpoint, failure/cancel/noop,
single-history and busy/stale/DPI/focus/resize/late-release rules remain.

Provenance: [PLAN-837](../../plans/archive/837-autographics-scene-hierarchy-and-group-authoring.md), [final R2](../../plans/attachments/837/review-r2/review.md), [merge receipt](../../plans/attachments/837/merge-r1/receipt.md). Source integration remains autographics-dev; production I2/I3 pending.
