# Scene document — PLAN-835 experimental contract

Status: delivered experimentally after final R2 and frozen closeout gates; independent Windows authoring experiment on autographics-dev. I1 real Auto games stay delivered; I2 production AutoUI/RQHost and I3 Forge remain pending.

`authoring::document::Document` original PLAN-835 schema used strict UTF-8 JSON `autographics.scene/1`; the hierarchy schema2 is historical; current schema3 is specified in the PLAN-838 extension below, retaining nonempty document_id (≤128 Unicode chars), fixed ordered resources `[circle,quad,cylinder,player,mob]`, and an ordered objects array. Object fields: positive unique u64 id, name≤128 Unicode chars, lowercase kind, translation[3], scale[3], yaw, pitch, rgba[4], visible boolean. The resource kind is a fixed registration identity, never an arbitrary file path.

Supported kinds: circle/quad (2D); cylinder/player/mob (3D; original procedural cylinder or unchanged original GLBs/all meshes/materials). Finite transform values have magnitude≤100000; scale>0; RGBA0..1. 2D requires z0, scale-z1 and yaw/pitch0; 3D retains fixed rgba `[0.9,0.3,0.2,1]` placeholder and original model material. Canonical encoding is serde struct field order with finite f32 values. Equivalent whitespace is accepted on input; semantic equality and canonical-content SHA256, not input byte formatting, define document/dirty identity.

Max128 total groups and objects; encoded file≤1MiB, input bounded before parse. Duplicate JSON keys (including nested objects), duplicate/zero object IDs, unknown schema/fields/kinds, type mismatch, bad numbers, invalid fixed resource manifest, illegal dimensions or limits fail before consumer use. No silent truncation. Document decode/validate/encode have durable accepted/invalid controls in `authoring_document`.

Persistent IDs are distinct from current epoch/revision and VM/runtime/resource ownership. Selection, drafts, focus, history, GPU pointers, session IDs, callbacks and game state never persist. New/Open rebuild current runtime data; no old message may mutate the new epoch. Same-session created IDs use a monotonic high-water mark; deleting/undoing does not revive old VM/target ownership.

Storage is `authoring::storage::Storage`, with bounded reads and a full validation/preview preparation before Open publication. Bad/missing/unsupported files keep accepted document/history/selection/dirty/preview. Dirty Open/New/Close uses explicit Save/Discard/Cancel. Save snapshots accepted epoch/revision/content/hash, explicit target and expected original bytes; pending edits block save. Save As confirms the new path only after successful readback. Parent directory must exist.

An exclusive owned create_new sidecar rejects occupied files without stale-lock deletion or best-effort fallback. A unique same-directory create_new temp receives complete bytes, flush and readback; expected original bytes are rechecked before rename. Rename is the commit point. Precommit failure preserves target bytes. The postcommit verification receipt stays pending on read failure, reports uncertainty and leaves dirty/path unconfirmed; verify/retry recognizes its own committed bytes and heals rather than declaring a permanent conflict. Foreign locks/temp files are untouched. Old-epoch snapshots cannot write or confirm a new document; saving an earlier same-epoch snapshot preserves dirty for newer current content.

Scope: local Windows same-volume single writer; no guarantees for power loss, arbitrary network filesystems, or noncooperating writers after the final comparison. Tests `authoring_storage` cover real precommit failures, committed-but-unverified retry, external mutation, actual Windows deny-share occupancy/recovery, foreign paths and old snapshots. `autographics-authoring --inspect file.agscene` reconstructs and renders in a new process.

Delivery: [PLAN-835](../../plans/archive/835-autographics-scene-authoring-save-and-undo.md), [review R2](../../plans/attachments/835/review-r2/review.md), [merge receipt](../../plans/attachments/835/merge-r1/receipt.md). Production I2/I3 remain pending.

## PLAN-836 viewport extension

Historical PLAN-836 retained schema1. PLAN-837 below supersedes the save schema and node budget; the remaining persistence rules continue. Pointer/camera/gizmo/gesture/sample/candidate ownership never persists. Save only consumes accepted Idle content; active gesture Save is rejected before Storage. Successful gesture is one before/after history step with ordinary hash-based dirty/Undo/Redo. Cancel/failure does not modify saved bytes. Reopen reconstructs saved content with clean dirty, no selection or history.

Provenance: [PLAN-836](../../plans/archive/836-autographics-viewport-picking-and-translation.md), [final R2](../../plans/attachments/836/review-r2/review.md), [merge receipt](../../plans/attachments/836/merge-r1/receipt.md). Source integration is autographics-dev only; production I2/I3 remain pending.

## PLAN-837 historical hierarchy schema contract

PLAN-837 save schema autographics.scene/2 adds groups plus explicit nullable parent
to each group/object. Group fields id/name/dimension(2d|3d)/parent/translation/visible;
leaf fields unchanged plus parent. Unknown/missing/duplicate fields and invalid
forest fail before consumers. v1 strictly migrates into root leaves/empty groups,
same IDs/local fields/order, without rewriting the source. Storage retains raw
original bytes for conflicts; saved canonical v2 semantic hash makes import clean.
Explicit Save/Save As writes v2, derived values and UI/history/runtime never persist.
Old readers do not read v2. New-process inspect rebuilds profile4, clean with no
history/selection.1MiB/128 Unicode/64 history steps/16MiB budgets retain meaning,
but the128 budget now includes groups and leaves. No old leaf is dropped to add groups.
Original local single-writer owned lock/temp/commit/readback recovery remains.

Provenance: [PLAN-837](../../plans/archive/837-autographics-scene-hierarchy-and-group-authoring.md), [final R2](../../plans/attachments/837/review-r2/review.md), [merge receipt](../../plans/attachments/837/merge-r1/receipt.md). Source integration remains autographics-dev; production I2/I3 pending.

## PLAN-838 current static asset extension

Current schema is autographics.scene/3. Ordered fixed builtin resources remain.
Add assets array of {format:"glb",sha256:<lowercase64hex>,byte_length:positive
bounded integer,name?:<=128 Unicode chars}; duplicate identities reject. Every leaf
has an explicit nullable resource. builtin kinds require null; independent Model
kind requires sha256:<hex> declared in this document. The document remains strict,
bounded1MiB and128 groups+leaves, without derived/UI/history/runtime data.
v1/v2 strictly validate their own field surfaces, then map to empty assets and
resource:null; persistent IDs/order/locals/groups and rendered geometry preserve
meaning. Import is clean without writing input. Source raw bytes continue to
protect conflicts while saved hash uses canonical v3. Explicit Save writes v3;
old readers cannot read it. Open validates the full closure and prepares Auto/GPU
before one publication; any missing/hash/length/support/budget/preparation failure
preserves the old session. Post-entry unverified receipts retain snapshot assets;
Verify/retry checks entry and every declared blob before healing its own receipt.
Old epochs never write/confirm new documents; old revisions cannot clean newer edits.

Provenance: [PLAN-838](../../plans/archive/838-autographics-glb-import-and-portable-scene-assets.md), [independent final R2](../../plans/attachments/838/review-r2/review.md), [merge receipt](../../plans/attachments/838/merge-r1/receipt.md). Source remains autographics-dev; production I2/I3 pending.
