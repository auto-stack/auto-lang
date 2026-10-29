# Finite textured GLB model closures

The Windows x64 authoring driver adds an explicit static textured model profile
alongside legacy flat GLB and PNG sprite assets. Import accepts a glTF2 GLB with
one embedded geometry buffer, exactly JSON+BIN, one scene and static triangles.
The legacy geometry/node/accessor/span/sparse/finite-value guards still apply to
all declarations, including unused ones. POSITION/NORMAL and UV0 FLOAT or
normalized U8/U16 are supported. Fixed originals' unused TANGENT must be
non-normalized FLOAT VEC4, match position count, have checked storage, finite
components and w=±1; it does not enable normal mapping. Animation/skin/morph,
other vertex channels, compression, additional material channels and unsupported
extensions refuse by name. Original character animation is not removed to
fabricate static acceptance.

Materials are opaque, single- or double-sided, baseColorFactor multiplied by an
sRGB baseColorTexture in linear space; metallic0, roughness1 and no emissive or
other texture channels. All decoded model-image alpha values must be255. A
double-sided primitive disables culling consistently for colour and shadow,
and back-facing colour fragments invert their normal for lighting. Geometry
depth and the existing directional shadow pass remain real mesh passes.

Images are finite static PNGs in the existing1..2048/RGB-or-RGBA8/noninterlaced
domain, supplied by a checked GLB bufferView or bounded local relative URI.
No data/network/absolute URI, path traversal, percent/query/fragment variant,
Windows device path or link/reparse dependency is accepted. Declared image
sources are ordered and unique, including unused images. Driver admission
validates all GLB declarations and effective UV transforms before dependency IO
or a decoder; the general GLB importer receives admitted geometry-only scratch
with no image URI and performs no arbitrary dependency IO.

Only KHR_texture_transform on baseColorTexture is recognized: finite offset,
rotation and scale in the glTF order offset + rotation(scale × UV0), with
texCoord0. Transform overflow and missing/nonfinite UV0 refuse before decoding.
Nearest/linear magnification and minification, clamp/repeat/mirrored-repeat on
each axis are supported. Mipmap minFilter9984/9986 normalize to base nearest;
9985/9987 normalize to base linear. Textures have one actual mip level; retaining
sampler lambda for min/mag choice does not add levels or implement mipmapping.

The AssetRef retains original root SHA256, byte length and `glb` format, with an
optional closure record `{id,images}`. Each image edge records source (`uri:...`
or `view:<index>`), original image SHA/length and dimensions. Closure ID is the
SHA256 of compact UTF-8 serde JSON for the tuple
`("autographics.textured/1", root_sha, root_length, ordered_image_records)`;
image record field order is source,sha256,byte_length,width,height. Root bytes
already bind sampler/material declarations. Resource keys are `closure:<id>`;
legacy resources keep `sha256:<raw_sha>`. Same root with different external
images may coexist; original root bytes and URI strings are never rewritten.

Accepted Asset owners retain immutable root and external PNG bytes plus shared
decoded image Arcs. SaveAs uses these accepted bytes, not the import directory.
Scene6 binds closure metadata in the hashed document entry; schemas1..5 reject
the closure field, including null, before upgrade. Canonical storage remains
`assets/<raw_sha>.glb|png` with explicit per-model edge mapping and exact source
blobs. Embedded image raw bytes remain part of the GLB, not another disk blob.
Open and pending-save Verify recheck the complete root/dependency binding even
when models are cached. Project/entry locks, immutable no-overwrite install,
entry-last commit, readback confirmation and residual/retry rules are inherited;
missing, changed or foreign blobs refuse and restoring exact bytes recovers.

All roots and independent PNGs share the existing8 original identities/32MiB
raw union across current/history/pending owners; each raw is≤8MiB, GLB JSON≤1MiB.
Embedded PNG raw is not charged twice, but all decoded images count. Full raw
quota still permits streaming exact duplicates without another retained snapshot.
CPU and expected GPU each remain≤64MiB, now including UV arrays, conservative
edge/material/node metadata and actual image storage. CPU images and raw bytes
share real Arc owners. Model meshes share one same-device GPU image by source
SHA; a PNG used by both separate2D and3D stores reserves both physical textures.
Prediction precedes decode; actual uploaded vertex/UV/aligned-index/texture bytes
are checked against reservations. Original node/primitive/vertex/index/mesh512
limits and user256KiB/managed96KiB/combined352KiB,32credits,VM timing limits remain.

Each expanded primitive retains original node, mesh and primitive indices and
invertible node-world transform beside effective UVs and the immutable original
GLB. Source-local geometry remains recoverable for006; this is not an animation
or pose API. UV/material changes leave actual collision AABB coordinates intact.

GPU caches are private staged views until whole-frame acceptance. Failure before
or after resource preparation preserves the complete old tick, role/object set,
camera and pixels. Warm motion/camera steps do not reparse or reupload static
resources. Public GpuDevice admission binds the entire store to the host device
generation before key lookup, including a new resource key; raw Iced admission
remains explicitly caller-owned/unattested. Stop releases Run owners while the
editor legitimately retains assets. Revoking payload ownership also retires its
draws; missing meshes of a still-owned payload remain renderer errors. Actual
Weak owners prove final CPU/GPU release after legal external pins are dropped;
driver-internal residency and GPU timestamps remain excluded.

Evidence: pinned Kenney3fa8a04 platform/coin/flag + PNG, public bound scene,
relocated save/open/Run, raw/decoded quotas and declaration negatives,36 sampling
cases/144 CPU-vs-GPU probes, depth/back-face/shadow controls, real UV/transform
mutations and20 real Auto object/camera Runs with zero warm static uploads and
actual last-owner release. Source/runner/report identity and fresh offline SDK
consumer are bound by PLAN-005's delivery receipt. No full game, Gallery package,
full PBR, character animation, production UI fusion or new platform is implied.

Provenance: [auto-ui PLAN-005 r3](../../../../auto-ui/docs/plans/archived/005-autographics-textured-models-and-uv.md), independent final R3 pass reviewed `74378791b353cd285168fa43fb7fe4b27cd208fe`; delivery `351919c76b4ef1d940fc7b8120254e321c41cba2` (fully equal rebase, unchanged experimental tree). Frozen SDK and both Cargo locks unchanged. Full AG milestone remains pending.
