# PLAN-838 proposed canonical delta, revision1

Proposal only. Final review binds this file to the frozen code and actual cases;
merge publishes canonical knowledge after its complete gates. Base is delivered
PLAN-837 cf2e99f50ee2a70b70ca2cfc23a5af56b405638b. I1 remains the original two
Auto games; I2/I3 and production integration remain pending.

## SD-01 add docs/specs/auto-graphics/scene-assets.md

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

## SD-02 modify scene-document.md

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

## SD-03 modify scene-hierarchy/authoring-session/viewport-interaction.md

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

## SD-04 modify auto-source-driver/prototype-profile/project/integration-contract.md

Keep profile1/2 games and profile3/4 sample contracts. Asset documents explicitly
use new samples/authoring_assets/main.at profile5. Auto accepts parent-first
hierarchy plus candidate-local resource ordinals, derives world/effective state,
and returns every node's kind/parent/dimension/ordinal and complete fields. Rust
checks exact count, identity, resource mapping and fields, then consumes the actual
returned resource/world/effective data for rendering/pick. Wrong resource output
is a whole-candidate rejection. GLB decode and GPU ownership remain in the host.
The same pinned SDK9576b4d and independent two workspaces/locks remain, with the
driver's direct locked gltf1.4 dependency and extras feature recorded in source.
Current source identity is plan838-identity.json; .agscene text hashes normalize
LF/CRLF, fixing fresh-checkout reproduction of inherited837 fixture bytes.
CLI adds --asset-check/--asset-file/--asset-save/--wrong-asset/--asset-gpu-fault
and retains --drop-release/--frames/--inspect. Native/report/aggregate completeness
and actual content controls remain mandatory. The slice provides portable finite
scene/static-model authoring. Production I2/I3, general materials/animation,
multi-platform/public engine/Forge distribution remain pending.
