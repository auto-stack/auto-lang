# PLAN-838 work handoff, revision1

Authoritative tree D:/autostack/.wt/lang-838/auto-lang, plan-838-dev; base is the
delivered837 cf2e99f50ee2a70b70ca2cfc23a5af56b405638b. SDK remains
9576b4dad5ceb8b201adedd46f9689f25c2dd5f9 with680 inputs/17 local packages/29 edges
and479-package SDK metadata closure. Exact final commit, source/lock hashes,
commands/results and raw evidence live in the shared PLAN-838 work receipt.
This is implementation-context checking; final independent review remains next.

## Current contract and evidence entrypoints

| AC / V | Concrete producer/consumer and distinguishing assertion |
|---|---|
| AC01/V01 | Delivered837 archive+complete-gates+actual branch/canonical/cleanup/not-due; pinned SDK and two independent locks; verify_source + verify_plan838 source/own-GLB/original-GLB/lock controls; receipt-driven independent source export/own target |
| AC02/V02/03 | assets::read_for_import bounded single read or streaming duplicate comparison; strict unique GLB JSON -> gltf no-IO validation/readers -> AGP loader -> complete array validation; sparse/U8/U16/U32/nonindexed accepted controls, unsupported declarations/URI/material/graph/span rejection before decode counter; exact65535/65536,65536/65537 vertices,16/17 parts,128/129 nodes,262144 expansion,8MiB/1MiB/32MiB and predicted GPU union; hidden513 rejects while512 renders |
| AC03/V04 | Strict v3 assets + explicit leaf resource; v1/v2 field-surface validation then lossless migration with raw-byte conflict protection; wrong/duplicate/unresolved/kind resource rejection, old file not rewritten |
| AC04/V05 | Real profile5 resource ordinal/world/effective return -> full admission -> actual object.resource -> same-device staged store/full model renderer/OBB/group bounds; wrong Auto resource rejected; real original-last-index65535 GPU triangle, padding and device mismatch controls |
| AC05/V05/06/10 | Iced Import/Create/Replace and stable ID/parent/all local properties; one history step, same SHA same Asset/mesh Arc and zero extra decode/upload, same name different bytes separate; import/cancel generation and captured selection reject canceled/stale-target envelopes on unchanged revisions; staged GPU-after-upload failure preserves doc/history/store/old Arc and still counts attempted writes; cancel releases actual VM/raw/model owner; native9 independent part/origin pixel states |
| AC06/V07/08/10 | Root lock then entry lock; blob temporary/readback/no-overwrite rename before entry; actual runnable old external-asset scene/bytes/GPU image survive pre-entry failures; correct residual retry; tampered postcommit closure remains unverified/path+dirty unconfirmed then heals; Save As/relocation/source+old-root removal/new-cwd process reopen; missing/bad blobs fail and restored bytes recover; reserved entry targets/reparse refusal |
| AC07/V01/03/09 | Original profile1/2/3/4 sources/game assets untouched.20 cycles include import/reuse/replace/undo/redo/cancel/GPU failure/real disk Open/Close, actual weak VM/Asset/raw/GPU owner release, legitimate external pin control, stable target identities, warm upload/decode conservation; actual owner-retention and physical reupload faults fail the same permanent test. Unverified-save-only owners remain budgeted/GPU-retained after redo discard, then release on full verification |
| AC08/V11 | README, asset-import-feasibility, self-authored deterministic MIT GLB, this handoff, scene-assets-spec-delta SD01..04, source/SDK/lock closure and exact frozen merge matrix |

Targeted formal Rust entries are authoring_assets (17 cases), affected old
authoring_document/hierarchy/host/interaction/session/storage (32 cases), native
completion unit cases, asset_feasibility scratch and AGP asset_resources. The
scratch helper's parent no-op and its actually executed child are distinguished
in receipts. Negative controls must reach the named actual assertion/oracle;
compilation, initialization or environment failure is not accepted as sensitivity.

Final R1 repair adds three permanent regressions: resolved Windows save-target
identity protects immutable blobs/locks through case/.. aliases and preserves
dirty/path/receipt state; semantic accessor types/dimensions/normalization reject
before all typed readers, including unused meshes and sparse payloads; raw morph
and matrix/TRS declarations reject before gltf convenience accessors hide them.
Invalid cases replay through production Host with old document/history/owners/
uploads/pixels preserved, then a valid import recovers. Valid interleaved/sparse,
FLOAT/normalized-U8/U16 UV, integer indices, matrix-only and TRS-only controls
preserve actual geometry and original bytes. Durable reviewer probes and current
closure rows/results are in shared attachments/838/work-repair-r1/readiness.md.

No open work-owned gap can be waived by a green total. Latest readiness must name
any gap explicitly; execution_done requires none. GPU error scopes cover new buffer
validation/OOM rejection; physical device loss is not a recoverable device-recreation
feature. Synchronous parsing/compilation/native calls are measured separately,
not claimed preemptible. JSON/parser temporary metadata, driver residency, Iced
internal buffers/atlas and GPU timestamps are outside final geometry measurement.

## Source and profile changes

Only this workspace and AGP's pipeline.rs/local resource test change. Driver adds
direct locked gltf1.4 with extras, reusing the existing registry package; AGP's
independent manifest/lock, SDK files, original game samples/assets/goldens and root
workspace/lock stay unchanged. New profile5 coexists with the original sample
exports. Schema3 migration preserves semantics and retains original file bytes for
conflict checks. Raw source paths never persist or become a fallback.

Inherited837 hierarchy fixture was checked out LF while its old receipt recorded
CRLF. Exact CRLF reconstruction passed all247 old source hashes and controls before
implementation. New source hashing normalizes .agscene text, including fresh
exports, without changing the old historical identity file. Package export copies
only receipt-listed source plus680 pinned SDK inputs, excluding target/reports.
The consumer verifies/builds with its own target and no producer compiled artifacts.
It does not inherit the producer root .cargo/config.toml. Before its runtime tests
or native build, set RUST_MIN_STACK=16777216 and
RUSTFLAGS="-C link-arg=/STACK:33554432" explicitly. These are the existing pinned
SDK's separate worker-thread and Windows PE stack requirements; scene data does
not depend on environment settings when opened by the correctly built program.

## Work checks

```powershell
$env:RUST_MIN_STACK="16777216"
$env:RUSTFLAGS="-C link-arg=/STACK:33554432"
cargo check --locked --all-targets
cargo fmt -- --check
cargo test --locked --test authoring_assets --test authoring_document --test authoring_session --test authoring_storage --test authoring_hierarchy --test authoring_interaction --test authoring_host -- --nocapture --test-threads=1
cargo test --locked --bin autographics-authoring -- --nocapture
cargo test --locked --test asset_feasibility -- --nocapture --test-threads=1
cargo test --locked --manifest-path ../autographics-prototype/Cargo.toml --test asset_resources -- --nocapture
python tools/check_asset_owners.py research/asset-owner-NEW
cargo rustc --locked --offline --bin autographics-authoring -- -C link-arg=/STACK:33554432
python tools/check_scene_assets.py research/asset-native-NEW
python tools/verify_source.py
python tools/verify_plan838.py --controls
python tools/package_consumer.py D:/autostack/.wt/lang-838/consumer-NEW
```

Native positive requires phase20,9 pixel-state records with all four independent
part/origin assertions, actual saved entry+resources and a child reopen from a
relocated directory with its source and original root removed. Negatives are
wrong-resource, missing-part, transparent, wrong-GPU-model, drop-release timeout
and frame1 incomplete. Native exit/report and aggregation must all agree.

## Review/merge-owned gates pending

Final review inspects this same frozen tree; implementation pauses. A pass alone
does not land or deliver the Plan. Merge then runs the complete locked driver and
AGP test/builds,16 AGP cases and both composition counter/content controls, original
two-game/835/836/837 native positives/negatives/completeness, current assets native
and owner controls/source/independent fresh-consumer gates. Use fresh output paths.

From driver, after final pass:

```powershell
$env:RUST_MIN_STACK="16777216"
cargo test --locked -- --test-threads=1
cargo build --locked --bins
cargo rustc --locked --offline --bin autographics-auto-driver -- -C link-arg=/STACK:33554432
cargo rustc --locked --offline --bin autographics-authoring -- -C link-arg=/STACK:33554432
python tools/verify_source.py
python tools/verify_plan838.py --controls
python tools/check_hierarchy.py research/merge838-hierarchy
python tools/check_hierarchy_pixel_faults.py research/merge838-hierarchy-pixels
python tools/check_scene_assets.py research/merge838-assets
python tools/check_asset_owners.py research/merge838-owners
./target/debug/autographics-authoring.exe --window-probe --report research/merge838-legacy.json
./target/debug/autographics-authoring.exe --window-probe --sabotage-target --report research/merge838-legacy-wrong.json
./target/debug/autographics-authoring.exe --interaction-check --report research/merge838-interaction.json
./target/debug/autographics-authoring.exe --interaction-check --sabotage-target --report research/merge838-interaction-wrong.json
./target/debug/autographics-authoring.exe --interaction-check --drop-release --report research/merge838-interaction-drop.json
./target/debug/autographics-auto-driver.exe --window-probe --report research/merge838-game1
./target/debug/autographics-auto-driver.exe --window-probe --sabotage-focus --report research/merge838-game1-wrong
./target/debug/autographics-auto-driver.exe --sample squash --window-probe --report research/merge838-game2
./target/debug/autographics-auto-driver.exe --sample squash --window-probe --sabotage-focus --report research/merge838-game2-wrong
python tools/package_consumer.py D:/autostack/.wt/lang-838/consumer-merge838-NEW
```

Each sabotage/drop command must fail its actual oracle. The independent consumer
then verifies source/controls, all-targets checks, scoped assets tests, native32MiB
build and check_scene_assets outside producer. AGP complete commands/16 IDs and
AGP_SABOTAGE_ICED blank_blit/transparent_blit counter+content controls are exactly
the inherited hierarchy-handoff.md matrix, refreshed to the frozen838 commit
(AGP_BASE_COMMIT for independent-consumer). Both content AND counters remain gates.

Production root Rust/schema/UI generators are untouched: root t/tv/tt/tb/tu/taa
and docs_gen are not work gates here. tf is main-only batch regression when due.
T-09 independent review and T-10 frozen complete gates/actual autographics-dev
landing/canonical SD+ledger/archive/guard cleanup/batch-due receipt remain pending.
Keep this tree. No new chat/agent or automatic review/merge was authorized by the
work-only request.
