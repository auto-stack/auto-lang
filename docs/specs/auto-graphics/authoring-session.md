# Authoring session — PLAN-835 experimental contract

Status: delivered experimental; stock Iced, one Windows window, two fixed2D/3D previews. No production UI/RQHost/Forge migration or game editing is implied.

`authoring::host::Panel` exposes real list/property/create/delete/undo/redo/open/save widgets through public SurfaceUi/IcedSurface. `authoring::session::Editor` owns accepted Document, Preview/VM, selection, epoch/revision, before/after history and saved content hash. Drafts remain UI-local; epoch/revision/target bind both draft callbacks and Apply. Selection changes, blur/deletion and new epoch reset drafts; text focus does not produce preview/game keys. Pending transactions are visible, block reentry/save/navigation, and share one content owner regardless of mount count.

Flow: widget command -> current stamp/target admission -> candidate document validation -> private profile3 VM with full config -> bounded scalar calls/output -> checked exact target/count/kind/visibility/finite property output -> original resource/GPU preparation -> publish document/preview/history together. Failure discards candidate, preserves last-good/selection/history, and the next valid operation must succeed. No hard-coded Rust draw fallback for failed Auto. Rust only marshals and presents actual Auto values. `preview::Building` pumps at most8ms per event-loop round using inherited public CPU slices; signature/type/budget faults terminate only the candidate. Compilation remains synchronous (measured separately), with inherited trusted-source/native-call limitations. Every edit creates a new VM; rejected/old owners release via actual Drop/close, not counters.

Profile3 public Auto exports: reset, identity(int ordinal,int kind,int visible), append_field(float), profile/count/id/kind/visible/field queries. Full checked objects map stable document IDs to transaction-local positive ordinals. Profile1/2 exports and default driver entry remain unchanged. `samples/authoring/main.at` is the source; frozen SDK9576b4d is read-only. 2D consumes DrawCmd, 3D consumes all original ModelData mesh parts on the same device/queue as Iced, through SurfaceRuntime. Selection has yellow2D bounds/projected3D bounds, separate from saved content.

Successful non-noop commands increment revision; rejects/noops do not. Undo/redo restore exact before/after content, check current stamp, increment revision, and new edits after undo discard redo. History≤64 steps and≤16MiB summed retained canonical before/after bytes; accept then evict oldest whole steps until both limits hold. A candidate step over budget is refused. Input drafts and rejected commands never enter history. Reopen clears history and creates a new epoch. Dirty compares current canonical SHA256 with last confirmed saved content hash, independently of revision/history cursor/eviction. Undo back to saved content is clean; saving an older snapshot cannot clear newer content.

`storage` handles the local single-writer protocol in scene-document.md. Open/New/Close while dirty or unverified offers Save/Discard/Cancel; Save failure/unverified result cannot close/discard. Cancellation preserves accepted content. Open validates/loads/prepares before replacing; old callbacks/save completions cannot select/change/clean the new epoch.

Acceptance targets: authoring_document, authoring_session, authoring_storage, authoring_host, authoring_feasibility. Controls cover bad Auto output, stale target/revision/epoch/draft, resource preparation failure/recovery, duplicate pending delivery, same-name object isolation, history/dirty branches/budgets, actual widget create/delete/apply, shared mounts once, upload omission,20 actual VM/runtime/mesh last-owner cycles plus legitimate pins. Native --window-probe observes real OS messages through winit to Iced controls and actual GPU captures; --sabotage-target must fail that same gate. New-process --inspect confirms persisted scene reconstruction and empty history/clean dirty.

Measured counters expose real target bytes/count, mesh buffers/count, scene/2D pipeline buffers, depth textures, actual write_buffer calls/bytes. Runtime content and composition upload counters must sum; omission is rejected. Warm target identity is retained. Reports state exclusions for Iced internal/atlas/driver allocations and unavailable GPU timestamps. CPU frame P50/P95, synchronous compile time and Windows working set are measured, not GPU time claims.

Complete driver/AGP gates and landing stay merge-owned after independent final review. Production I2/I3, arbitrary UI trees, parent/reparent/Portal editing, rotation/scale gizmos, free resources/materials, collaborative/network persistence and crash-durable device flush are outside this finite experiment.

Delivery: [PLAN-835](../../plans/archive/835-autographics-scene-authoring-save-and-undo.md), [review R2](../../plans/attachments/835/review-r2/review.md), [merge receipt](../../plans/attachments/835/merge-r1/receipt.md). Production I2/I3 remain pending.

## PLAN-836 viewport extension

Idle/armed/dragging/finishing lifecycle binds gesture token, target, epoch/revision, captured Surface/path, press position and actual camera. A gesture owns1 inflight Building,1 latest coordinate,1 displayed private Preview; Editor retains accepted Preview. Full candidate document flows through real Auto profile3 and exact checked output/resource admission before any temporary renderer publish. Accepted Document/hash/history/revision stays unchanged until successful last release endpoint, which uses ordinary history publication once. Repeated release and final noop never add history. Candidate fault/invalid/stale/cancel restores accepted Auto output without recompilation/model upload; following legal gesture can recover. Late up/move/results cannot commit to a new target. Esc/blur/leave/resize/DPI/zero-size or camera changes cancel capture. Busy commands/drafts are discarded rather than queued; Close cancels then reuses the existing accepted-content unsaved decision. Inherited8ms pump/cumulative call budgets remain; compile is synchronous. Finishing deadline30s is a controlled rollback, not a UI frame latency promise. Fixed7 mesh resources are prepared once on this device; per-gesture candidate publication never recreates store/pipeline/target. Actual weak VM owners, unchanged GPU Arc identity/upload writes, target identity and external last-owner pins establish lifetime bounds; metric exclusions stay explicit.

Provenance: [PLAN-836](../../plans/archive/836-autographics-viewport-picking-and-translation.md), [final R2](../../plans/attachments/836/review-r2/review.md), [merge receipt](../../plans/attachments/836/merge-r1/receipt.md). Source integration is autographics-dev only; production I2/I3 remain pending.

## PLAN-837 current hierarchy contract

Replace/CreateGroup/Reparent/Delete-subtree are stamped transactions. Target identity,
selection and expected old parent gate structure commands. Reparent preserves only
world position: old world minus new parent world, unchanged descendant local fields
and own visible; effective visibility recomputed. Reject cycle/cross-dimension/leaf
parent/missing/bad numeric candidate. Recompute target+descendants and reject if
any coordinate moves beyond max(1e-3,8*f32EPSILON*max(1,abs(before),abs(after))).
No descendant compensation, clamping or relaxed precision. Group delete UI shows
descendant count and confirms; cancellation changes nothing. Undo/redo restore exact
subtree IDs/local fields/order in one before/after step. New edit clears redo;
created ID high-water does not retreat on delete/undo and exhaustion only prevents
new creation. Selection/drafts reset as before; stale target/parent/epoch/revision
or pending structure commands cannot mutate current content. Source profile3 remains
available as the unchanged original sample; current new/import editing explicitly
uses profile4. It never guesses profiles from payload shape.

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

## Temporary external source selection (PLAN-840; retained beside project defaults)

Editing supports Builtin6 or Use user script plus one .at path. Ordinary CLI: autographics-authoring --open C:/project/scene.agscene --behavior main.at. Relative paths resolve against actual saved scene Storage root; unsaved scenes need an absolute path. Save As changes this root, with no fallback to the former root or Builtin6. Starting/Running/Paused/Faulted reject source/path changes without queueing. Each Run reads strict UTF-8/no-BOM regular local source, validates and compiles an immutable snapshot, and shows source/API/hash diagnostics; fix the file, Stop and Run to recover. This temporary selection is not persisted. Scene4 separately stores an explicitly accepted binding as described below. [Behavior API](behavior-api.md) defines limits and callbacks.

Provenance: [PLAN-840](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md), [affected final review](../../plans/attachments/840/review-r2/review.md), [complete merge receipt](../../plans/attachments/840/merge-r1/receipt.md). Graphics source: autographics-dev. Production I2/I3 remain pending.

## Bound-source editing (PLAN-845)

Bind/update, role changes and detach are accepted candidate transactions; Document+CodePayload+prepared preview publish together. Current/pending/history/unverified snapshots share immutable owners. Across them admit at most8 distinct identities/2MiB raw source bytes, separate from existing64step/16MiB history. Exact raw bytes survive original-file removal and Undo/Redo. Noop/rejection/stale/cancel or failed preview preparation preserve accepted binding/dirty/history. Bound-reference deletion rejects until detach/retarget; explicit nullable roles are stable same-dimension leaf IDs.

Default Run uses accepted binding/roles; explicit external/Builtin is a temporary override only. New/Open resets that override. Pending receipts and editing/draft/file operations retain their original busy admission. Successful Load/complete Verify records bounded current-root source identity provenance, independent of overall document dirty; unconfirmed source diagnostics use memory/hash. Root-changing SaveAs cannot carry old source proof and old-revision receipts confirm their own source. [Behavior projects](behavior-project.md) owns the complete rule.

Provenance: [PLAN-845](../../plans/archive/845-autographics-bound-behavior-and-portable-projects.md), [independent final review](../../plans/attachments/845/review-r8-preparation/review.md), [merge receipt](../../plans/attachments/845/merge-r1/receipt.md). Source: autographics-dev; production I2/I3 pending.

## Current editor host responsibilities (PLAN-855)

The public authoring::host::Host remains the sole Editor/Play/runtime/resource composition. host.rs retains its original fields, construction and resource/probe introspection. Its private child impls split existing responsibilities: host/panel.rs owns stock Iced widget presentation and local draft/stamped-message methods; host/commands.rs owns editor/project dispatch, accepted-only persistence and unsaved decisions. No new mutable public state, adapter storage, event queue or second command handler is introduced.

The actual flow remains Panel stamped message -> Host admission -> same Editor candidate/accepted/history plus Storage receipt -> resource preparation -> one accepted publication. Editor owns accepted/pending/history and immutable code/assets; Storage owns root/receipt/disk protocol. Busy, stale, cancellation, failed preparation and late-result behavior, dirty/history/save/reopen semantics and legitimate external pins retain the existing contracts. This is implementation placement, not a new author API or transaction boundary.

Provenance: [PLAN-855](../../plans/archive/855-autographics-authoring-responsibility-split.md), [independent final R3 review](../../plans/attachments/855/review-r3/review.md), [merge receipt](../../plans/attachments/855/merge-r1/receipt.md). Source: autographics-dev at 05fdc81c546aff735eefdb6c39fbc5fb35dc546a; SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467. M2/S0 responsibility and entrypoint maintenance; new M2 capabilities and production I2/I3 remain pending.

## 2D editing view lifecycle (PLAN-856)

Editing View2d and the presented revision are owned by the same Host. View, grid and snap settings are not Document/scene4 data, history or dirty state. Navigation and object candidates are mutually exclusive; a continuous navigation cancellation restores its admitted starting view and 2D frame even when preparation fails, without acquiring another VM/mesh owner. Late input cannot publish. Atomic wheel completion survives later blur. New/Open success resets, failure retains; Save/Undo/Redo retain the view. Play and Starting use the default runtime view with navigation disabled; Stop restores retained editing content/view. Still-applicable unsubmitted property drafts remain intact during view changes. See [editor-viewports](editor-viewports.md) for exact bounds and input rules.

Provenance: [PLAN-856](../../plans/archive/856-autographics-2d-viewport-grid-and-navigation.md), [independent final R2](../../plans/attachments/856/review-r2/review.md), [merge receipt](../../plans/attachments/856/merge-r1/receipt.md). Source: autographics-dev at `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`; SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`. Production I2/I3 and 3D editor navigation remain pending.
