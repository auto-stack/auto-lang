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

During play, canvas presses select keyboard focus only; no selection/gizmo/gesture
starts. CompositionInput is authoritative for2D/3D/panel keyboard target. Held axes
and one toggle parity are sampled at dispatch, not simulated in Rust. Panel text/
buttons produce no actor actions. Canvas changes clear old held/parity; blur/leave/
resize/DPI/occlusion/minimize/Pause/Stop/Fault clear held/edges/debt. A Starting
cancellation survives all remaining initialization queries/preparation. Nonrepeat
new press is required after clearing and Resume; OS repeat cannot resurrect held input; keyup
still clears state after focus changes. Esc is handled once by the mode as Stop,
returning to Editing without another widget/game delivery. Bounds used by actual
OS probes account for Panel's actual clamped scroll translation; no fixed-coordinate scan is an
acceptance substitute.

Provenance: [PLAN-839](../../plans/archive/839-autographics-scene-play-mode-and-auto-runtime.md), [independent final R2](../../plans/attachments/839/review-r2/independent/review.md), [affected independent R3](../../plans/attachments/839/review-r3/independent/review.md), [merge receipt](../../plans/attachments/839/merge-r1/receipt.md). Source remains autographics-dev; production I2/I3 pending.

## Current Host input and publication placement (PLAN-855)

Picking/gesture primitives and state remain in authoring/interaction.rs. Host pointer/key/blur/resize/camera/scroll routing, capture/cancel, real Auto gesture candidate preparation, frame presentation and publication now live in host/viewport.rs. They consume the same Surface/presented snapshot and Host fields; there is no duplicate input state or new public gesture API.

The epoch/revision/target/token contract, duplicate release/noop, busy/stale/late result rejection, Esc/focus/DPI/resize/Play switching and accepted-only history publication remain unchanged. Native wrong-target phase1 and missing-release phase4 negatives retain their original oracle. Grid and camera navigation remain pending; this move adds neither.

Provenance: [PLAN-855](../../plans/archive/855-autographics-authoring-responsibility-split.md), [independent final R3 review](../../plans/attachments/855/review-r3/review.md), [merge receipt](../../plans/attachments/855/merge-r1/receipt.md). Source: autographics-dev at 05fdc81c546aff735eefdb6c39fbc5fb35dc546a; SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467. M2/S0 responsibility and entrypoint maintenance; new M2 capabilities and production I2/I3 remain pending.

## Current 2D editor view (PLAN-856)

The historical fixed mapping above remains the reset default. The finite [2D editor viewport](editor-viewports.md) now binds drawing, exact reverse-order ellipse/quad picking, group bounds/origin, selection and real Auto gesture candidates to the same presented View2d/revision. Pointer delta is converted once to child logical coordinates, while handles and the drag threshold retain logical-pixel size. Absolute final world origin is snapped before the parent translation is subtracted; group movement changes only that group local. Accepted-only publication, single-history/no-op, scene4 persistence, camera/stamp/selection checks and all earlier cancellation controls remain. Normal release retains the actual 2D keyboard target; true cancel/focus loss clears capture appropriately.

Provenance: [PLAN-856](../../plans/archive/856-autographics-2d-viewport-grid-and-navigation.md), [independent final R2](../../plans/attachments/856/review-r2/review.md), [merge receipt](../../plans/attachments/856/merge-r1/receipt.md). Source: autographics-dev at `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`; SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`. Production I2/I3 and 3D editor navigation remain pending.
