# Bound behavior and portable projects

The finite Windows authoring experiment stores one accepted Auto source and stable
actor bindings with the scene. Source lives on autographics-dev. It is trusted
local code under [behavior/1](behavior-api.md), not an untrusted sandbox, installed
engine SDK, multi-module package or production AutoUI/RQHost integration.

## Scene and source identity

`autographics.scene/4` requires `behavior`: null or an object with exact `api`,
`source` and `actors`. API is explicitly `autographics.behavior/1`, `/2` or `/3`. Source has
`format: auto-source`, a lowercase 64-hex SHA256 and positive `byte_length` at most
256KiB. `actors.two_d` and `actors.three_d` are explicit nulls or existing,
same-dimension leaf u64 IDs, including values above32bits. No group, missing,
cross-dimension or implicitly selected actor is admitted. Runtime ordinal handles
are generated at Run and never persisted.

A project is one scene entry, declared `assets/<sha>.glb` and one declared
`scripts/<sha>.at`. Original import paths, runtime state and API/bridge overrides
are not saved. Strict v1/2/3 decode migrates to scene4/behavior:null in memory only;
scene files are rewritten only by explicit Save. Unknown/missing/duplicate JSON
fields, invalid source identities or actor references reject before consumption.

## Acceptance, history and ownership

Bind/update reads the regular local file once under the behavior/1 byte limits,
preserves exact UTF-8/LF/CRLF bytes and uses the same SourceSnapshot validator and
real compiler as external scripts. Invalid UTF-8/BOM, forbidden AST/signatures,
wrong API or compile failure reject the candidate. Compilation does not call
app_start/update: a statically accepted source that faults or loops at Run remains
savable and Open never runs callbacks.

Document, CodePayload and prepared preview publish as one accepted transaction.
Pending/current/history/unverified-save owners share immutable source payloads.
Across them at most8 distinct source identities and2MiB raw bytes are admitted;
equal hash/length still requires byte equality before reuse. Original64step/16MiB
history limits remain separate. Rejected/noop/canceled/stale candidates do not
mutate accepted binding, dirty/history or source. Bind/update/retarget/detach and
Undo/Redo preserve exact source bytes, independent of the original import file.
Deleting a bound actor or ancestor rejects until explicitly detached/retargeted.
New/Open/Close/eviction drop their actual owners; lawful external byte pins are
separate from CodePayload lifetime. No automatic old-source disk garbage collection.

## Complete-closure save and recovery

A Snapshot fixes accepted entry/code/GLB/stamp. The existing root lock and entry
lock protect bounded prepare/flush/readback/non-overwriting blob installation,
then one same-volume scene rename commits the entry. scripts has a separate
8file/2MiB budget including reservations/residuals; unknown or incomplete paths,
reparse ancestors and invalid immutable blobs are preserved and reported.
Scene destinations cannot replace blob/root/entry locks or reserved temporary paths,
including Windows case/dot aliases; reject before verifying an old receipt.

Before entry commit, failure preserves the old entry/path/saved hash; valid
installed unreferenced blobs are reported as bounded residuals and can be verified
on retry. After commit, pending verification retains the full Snapshot. Verify
checks entry bytes and every declared GLB and code blob before confirming saved
state. Same-epoch old-revision receipts confirm only their own snapshot hash/source;
newer edits stay dirty. Old epochs never confirm a new document. Save As exports
accepted bytes to a new root even if old imports/root are gone; no fallback read.
Wrong existing immutable bytes fail without overwrite. These guarantees cover a
single cooperating local same-volume writer, not malicious races or power loss.

## Run, overrides and diagnostics

Open validates the whole declared closure and prepares the preview before
replacing Editor/Storage. Failure preserves the accepted document/view/history.
`autographics-authoring --open <scene>` uses the accepted bound source/roles by
default, or Builtin6 for behavior:null. Explicit external `--behavior` and Builtin
selection are temporary overrides; they never change binding/dirty/Save. External
sources retain their bounded per-Run file read and scene-root path resolution.
New/Open clears temporary selection. Busy Run/Starting/Running/Paused/Faulted
reject persistent edits and file/source changes rather than queuing them.

Default Run consumes accepted bytes, checks binding/hash/length/API/roles, and
never rereads the project blob. Unsaved/unconfirmed sources are labeled
`<memory:SHA256>.at`; confirmed sources name their verified current-root blob.
Storage retains at most8 identity-only provenance records from successful Load or
complete Verify, without source owners. Geometry-only dirty edits do not change
source origin. Undo/Redo uses source identity; Save As verification clears old-root
proof, while a pending/failed save publishes none. Old-revision Verify confirms
its snapshot's source rather than a newer binding. Missing/evicted proof uses
memory conservatively. Origin records prior verified provenance, not present disk
readability: later disk changes are detected by Open/Save/Verify, while Run still
executes accepted bytes. SourceSnapshot, Last Run and actual callback diagnostics
propagate this path/hash consistently.

Original frame admission, callback/queue/time budgets, Starting cancellation,
Pause/neutral Step/Resume, sticky errors and Stop editing/GPU restoration remain.
Run failures do not fall back to Builtin or old code. Inspect reports schema,
binding/roles, closure_verified and clean/no-history after successful Open.

## Verified finite boundary

One runner executes saved Move/Patrol and source-only speed updates with actual
Auto traces, real OS bind/Undo/Redo/Save As and independent all-part GPU pixels.
Actual UI-saved output is moved away from its import/root and reopened in a new
process/cwd with only --open; no source path or behavior parameter is needed.
Complete compatibility retains original16 AGP cases, dual games, hierarchy/assets/
Play, counter AND composed-content, owner and independent own-SDK/target gates.
Original20-cycle VM/GPU/payload ownership and upload tests distinguish leaks and
reuploads from lawful pins; arbitrary user heaps/driver residency are not quotas.
No hot reload, multi-file module/package, async/FFI/import or production I2/I3 is added.


Provenance: [PLAN-845](../../plans/archive/845-autographics-bound-behavior-and-portable-projects.md), [independent final review](../../plans/attachments/845/review-r8-preparation/review.md), [merge receipt](../../plans/attachments/845/merge-r1/receipt.md). Source: autographics-dev; production I2/I3 pending.


## PLAN-846 finite spatial extension

Scene4 Binding.api accepts exact behavior/1, behavior/2 or behavior/3. Temporary CLI/UI selection stays explicit and does not override a project default. Same raw bytes may share their Arc, but admission credentials remain API-specific; same-byte API switching is a real history/dirty transaction. Failure preserves the previous binding/source; save/open/verify and raw closure contracts remain.

Provenance: [PLAN-846](../../plans/archive/846-autographics-spatial-queries-and-trigger-interactions.md), [final affected review](../../plans/attachments/846/review-r6/review.md). SDK/root and graphics delivery identities are recorded in the Plan consolidation receipt; production I2/I3 remain pending.

## Finite segment extension (PLAN-848)

Scene4 accepts exact behavior/1,/2,/3 bindings. Same raw bytes may share an Arc, while admission/compiled credentials remain API-specific. Switching API is a real dirty/history transaction; failed admission preserves the prior source/binding. Default binding and temporary CLI/UI overrides, Save/SaveAs/Undo/Redo/Open and verified source closure retain their existing rules. Older readers reject unknown v3 explicitly.

Provenance: [PLAN-848](../../plans/archive/848-autographics-segment-casts-and-nearest-hit-interactions.md), [independent affected final review](../../plans/attachments/848/review-r4-final/review.md), [frozen merge receipt](../../plans/attachments/848/merge-r2/receipt.md). Source: autographics-dev; production I2/I3 remain pending.
