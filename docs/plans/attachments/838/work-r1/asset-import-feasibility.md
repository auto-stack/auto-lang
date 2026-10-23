# PLAN-838 execution entry and bounded investigation

Base: cf2e99f50ee2a70b70ca2cfc23a5af56b405638b, the delivered PLAN-837
autographics-dev revision, not its earlier review candidate. PLAN-837 is archived
10/10 with frozen gates, canonical deposit, guard cleanup and batch not-due receipt.
Authoritative tree: D:/autostack/.wt/lang-838/auto-lang, plan-838-dev.
SDK: 9576b4dad5ceb8b201adedd46f9689f25c2dd5f9, 680 files; metadata closure
479 packages, 17 local packages, 29 path edges. Both independent locks unchanged.

T-01 observed a reproducibility defect in the inherited identity: the hierarchy
example.agscene is stored LF in Git but its identity records CRLF. Reconstructing
only CRLF gives exactly the recorded SHA256 1a6d26ccb594bea9e18a3103be3a7dacb1c2c26ad94b3d79e3d04c8910ef4c62.
With those original bytes verify_source passes all 247 hashes; source drift,
missing scene and missing original GLB controls reject and restored control passes.
This is a known text transport difference, not an unrecorded semantic input change.
838 must make its own source consumer independent of this checkout line ending.

## Current T-02 contract (open)

The real path is Iced Import -> bounded snapshot -> no-IO GLB preflight -> AGP
load_glb_bytes -> unique primitive ordinals -> candidate registry -> profile5
Session -> exact result admission -> candidate MeshStore -> renderer and full-model
OBB -> accepted document/history. Save holds immutable bytes, prepares assets under
a root lock, then calls existing Storage entry commit; Open verifies the full
closure before publishing. Existing Storage alone does not verify asset closures.

Compare two bounded GPU options: fork the map of Arc<GpuMesh> with a shared
attempt counter, or build a store from explicit resident pins. The former is a
minimal public seam and avoids rebuilding/uploading old buffers. No renderer core,
shader, SDK or root workspace changes are authorized. Before downstream work,
prove a multi-part self-authored GLB traverses actual Iced/Auto/GPU, grouping/move,
OBB, directory Save As and new-process reopen without its source path. Include
bad index, 512/513, GPU preparation rejection and asset-before-entry failure with
recovery. Scratch evidence is not completion of production admission/codec tasks.

Owners: accepted/history/candidate each retain immutable asset Arc values;
candidate GPU fork owns only its new buffers and shares resident Arcs. Failed
preparation must release new owners without changing the resident map. Actual
attempted writes remain visible after rejection. Last history/current/external
pin release must free payloads. Valid multi-part color/position controls must
distinguish missing parts, wrong asset, u16 truncation and silent instance break.

## T-02 result and implementation choices

`cargo test --locked --test asset_feasibility -- --nocapture --test-threads=1`
passed 2 parent cases plus the executed child-process case (the parent process's
portable_child is an environment-gated helper, not an additional portable run).
Raw output: asset-probe/probe-run.txt. The self-authored two-triangle GLB has
two differently translated mesh nodes sharing a mesh display name, red/green
materials, and three u16 indices per part. Real Iced click -> profile5 scratch
Session selected ordinal/world -> both GPU centroid color assertions -> full-model
OBB passes. Parent/local translation changes the actual image; relocated directory
with original file removed and different child cwd reproduces that GPU image.
An immutable blob prepared before entry commit preserves old entry bytes and is
reused by the next commit. Candidate buffer pins share Arc identities, old map stays
empty on drop, last-pin release frees buffers, and shared write attempts persist.
Index65536 rejects, potential512 accepts and513 rejects (formal production
admission/upload/render boundary assertions remain T-03/04). No formal UI/schema
or OS-native acceptance is claimed by this scratch probe.

Choose map-fork over rebuilding from individual pins: same resource ownership with
smaller seam; shared Arc<UploadCounters> records failed attempts. Expose actual
MAX_INSTANCES. Pad only GPU index buffer tail to four-byte alignment, retain draw
index_count: the legal one-triangle probe exposed the old six-byte write failure.

Preflight uses strict unique-key GLB JSON plus locked gltf 1.4's no-IO parser and
accessor readers, before AGP import_slice. Direct driver gltf dependency reuses the
existing package; no registry version change. Validate the entire declared graph,
buffer/accessor spans and capabilities before traversal/allocation. Decoder output
gets stable expanded primitive ordinals, rather than display-name keys.

Schema3 fields: assets[{format:"glb",sha256,byte_length,name?}], Model leaf
resource:"sha256:<hex>"; builtins have explicit resource:null. Old schemas reject
these fields before strict migration. profile5 adds configure/query resource
ordinal; groups use -1. Asset paths never persist. Source bytes are immutable Arc
payloads retained by registry/history union, candidate discarded on failure.

Lock order: project-root create_new .autographics-assets-lock before original
entry sidecar; never invert. Only generated assets/<sha>.glb, explicit owned temp,
no overwrites/GC. Snapshot asset closure prepares before existing Storage commit;
entry and full closure readback must confirm together. Verify replays full closure
before healing its own unverified entry. Disk count includes immutable residuals
and temporary files; reject unknown files/links rather than delete them.

Evidence boundary: bounded scratch proves local mechanism, not production
resource preflight, history, native interaction, publication faults or20 rounds.
Those remain named T-03..08/V-02..10 acceptance work.
Final review and all merge-owned complete gates are pending.

## Implementation reconciliation

The inherited AGP uploader already rejected indices above65535 before its cast;
838 retains that guard, adds pre-decoder/vertex-range admission and observes an
actual65535 GPU triangle plus65536 refusal. The newly exposed issue was six-byte
triangle index writes; tail padding fixes that without changing draw count.
The direct gltf dependency enables its locked extras feature so legitimate large
metadata remains bounded JSON rather than a parser-feature rejection. No registry
package version or SDK source changes. Both declared and expanded geometry are
budgeted, including unused declarations.
The inherited decoder requires an explicit default scene. The agreed single-scene
profile also accepts an omitted optional scene declaration: original no-IO
preflight chooses the sole scene, then bounded parser scratch makes that selection
explicit only for the old decoder. Decoder-input SHA is checked separately from
the persistent original-byte SHA; file copying/save/reopen preserve original bytes.

The actual active registry includes current/history, pending candidates and a
committed-but-unverified save snapshot. The latter can outlive a redo branch;
Host accounts and retains its CPU/raw/GPU owners until complete Verify/retry or
authorized session replacement releases the receipt. A permanent counterexample
discards redo, fills seven new identities and proves the pending snapshot still
uses the eighth slot; full verification releases its actual weak owners and
permits the next import while newer content remains dirty.

Entry-before-commit failure evidence now starts with a valid different external
asset scene: entry bytes, old blob bytes and actual reconstructed GPU image all
remain usable. The new document is dirty, so postcommit failed closure validation
also proves saved hash/path do not confirm early. Final closeout measurements and
current source/commit identities are recorded in the shared work receipt.
