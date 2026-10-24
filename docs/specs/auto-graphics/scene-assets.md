# Static GLB assets and portable scene closure

The independent Windows x64 authoring profile imports bounded static GLB2 through
the actual Iced path/button. GLB has exactly JSON+BIN, a single embedded buffer
and scene. No URI/image/texture, animation/skin/morph, compression or extension
requirements are admitted. The sole scene is selected even when its optional
default-scene declaration is absent; only bounded decoder scratch adds that
declaration, while persisted bytes, original length and original raw SHA stay exact.
All declared nodes/accessors/materials/primitives,
including unused ones, are checked before AGP import_slice. Duplicate JSON keys,
bad references/spans/sparse indices, cycles/multiple parents, nonfinite geometry,
invalid normals and non-affine/singular/mirrored transforms reject. Static
triangles support POSITION/NORMAL and optional unused TEXCOORD_0; U8/U16/U32 indices
must be <=65535 and in vertex range, or nonindexed triples are generated. Sparse
storage is checked before its readers. Before any typed reader, every declared
primitive's POSITION/NORMAL must be non-normalized FLOAT VEC3; TEXCOORD_0 must
be FLOAT VEC2 without normalization or normalized U8/U16 VEC2; indices must be
non-normalized U8/U16/U32 SCALAR. Sparse values inherit that semantic type and
their own index/value spans are checked. Mesh/node weights and primitive targets
declarations reject even if empty or unreferenced. Raw nodes reject simultaneous
matrix and any translation/rotation/scale declaration before transform selection;
matrix-only and TRS-only remain valid. Material is opaque/single-sided, no
textures, alpha1, metallic0, roughness1, emissive0. Missing material is white with
those fixed values. Arbitrary glTF metadata extras are accepted as bounded JSON;
their parsing cost is measured outside final geometry budgets.

Raw GLB<=8MiB, JSON<=1MiB, nodes<=128, expanded primitives<=16, vertices/primitive
<=65536, vertices/asset<=262144, indices/asset<=786432. Declared and expanded
geometry both obey those budgets. All payloads active in accepted/history/pending
states count once by original SHA: <=8 identities/32MiB raw; <=64MiB final CPU
positions/normals/indices and <=64MiB expected GPU vertex/aligned-index arrays.
Prediction occurs before decoder allocation and actual arrays are checked after.
An exhausted registry still permits duplicate raw content by one bounded streaming
comparison, without another retained full payload. All 3D leaf potential mesh
instances, including hidden leaves, must fit the public renderer capacity512;
overage rejects the candidate before GPU upload/publication.

Model identity is original complete raw SHA256, byte length and glb format. Display
names never bind resources. Expanded primitive ordinal gives a stable GPU key
with SHA, preserving repeated names and repeated mesh nodes. A candidate mesh map
shares immutable same-device Arc<GpuMesh> values and a single cumulative upload
attempt counter. New uploads remain private until publish; rejection drops only
the candidate's new owners. Validation/OOM error scopes reject failed buffer
preparation. u16 index buffers pad their tail to4-byte writes while index_count
preserves the original triangle count. Warm unchanged resources do not upload.
Legal external pins retain payloads until their final drop; current/history/
candidate last-owner behavior is observed with actual weak VM/Asset/raw/GPU owners.
Driver/Iced internal residency and GPU timestamps are explicitly excluded.

Project entry parent is the root. Only generated assets/<lowercase64sha>.glb paths
are persisted by implication; no source/import paths are stored. Project-owned
root lock precedes entry lock. Immutable blob preparation/flush/readback and
no-overwrite same-volume rename precede the original single entry commit.
Entry and complete declared closure must read back before path/saved hash confirms.
Correct blobs left before entry commit are bounded reported residuals and reused
on retry; unknown/partial/foreign data remains untouched and reports its path.
Disk budget8 files/32MiB includes existing blobs/residuals and temporary reservation.
No background deletion/GC or stale-lock guessing occurs. Reserved project-lock
and canonical asset-blob paths cannot themselves be scene entry destinations.
Classification uses the resolved destination, with Windows case equivalence;
existing targets resolve their final name, new targets resolve their parent.
Case and ordinary .. aliases cannot bypass reservation or existing-entry conflict
identity. An invalid reserved Save As is rejected before receipt verification,
entry writes or path/saved-hash confirmation.
Local disk paths (ordinary relative paths included) are checked for every
existing ancestor and Windows reparse attributes before canonicalization/read/write;
network/device roots are outside this profile. Power-loss and malicious concurrent
path replacement remain the inherited exclusions.

Evidence: authoring_assets17 cases, including final-R1 declaration/typed-reader
and resolved-save-target regressions, AGP asset_resources, native check_scene_assets
positive9 pixel states and6 expected failures,20-round owner/reupload controls.
The controls check actual rendered parts and vacated pixels; state/commands/hash
alone cannot close a content criterion. Source fence and source consumer bind
exact current inputs and independent target/SDK closure.

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

## Scene4 code and GLB closure (PLAN-845)

The existing GLB rules, budgets, root lock, one-entry rename and resource ownership remain. A save Snapshot additionally owns one accepted code payload; both code and GLB install/readback precede scene commit and both are checked before Verify confirms saved state. scripts has its independent8file/2MiB disk quota including reservations/residuals; source payload union has8identity/2MiB memory bounds. Entry cannot overwrite source/blob/locks/reserved paths through Windows aliases, even before an earlier receipt Verify. Postcommit source/GLB failure stays pending, not clean; precommit residuals remain explicit. SaveAs uses accepted bytes, not an import reread. [Behavior projects](behavior-project.md) specifies recovery and scope.

Provenance: [PLAN-845](../../plans/archive/845-autographics-bound-behavior-and-portable-projects.md), [independent final review](../../plans/attachments/845/review-r8-preparation/review.md), [merge receipt](../../plans/attachments/845/merge-r1/receipt.md). Source: autographics-dev; production I2/I3 pending.
