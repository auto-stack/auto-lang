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
