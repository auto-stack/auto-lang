# PLAN-839:r1 proposed Spec delta

This is the work-owned proposal. Canonical publication/overview/plans/goals/ledger
belong to merge after independent review and frozen complete gates. Existing
scene schema3, resources and profiles1..5 retain their contract. I2/I3 remain pending.

## SD-01 add auto-graphics/scene-play-session.md

Windows single-window stock Iced/wgpu authoring supports an explicitly finite
scene Run mode with one retained real Auto Session. Built-in profile6 source is
samples/authoring_play/main.at, not arbitrary user script loading or a saved code
binding. Local Document and Registry come from accepted in-memory content, including
dirty/unsaved imports. Actor choice is a stable same-dimension leaf ID, one2D and
one3D; default first eligible document leaf, absent dimension None. Empty worlds
are valid. UI names the chosen IDs and cycles eligible actors only in Editing.

Editing(no Play) → Starting → Running → Pausing/Paused → Stopping → Editing;
Faulted retains the last accepted runtime frame and closes the contaminated VM.
Window cancellation during Starting latches a pause request and reason, clears
input/clock and finishes initialization under the same30s budget. Only the complete
validated/prepared initialtick0 is published, entering Paused directly. It dispatches
no background ticks before explicit Resume. Starting Resume/Step stay rejected;
Stop and initialization faults still release the runtime VM. Later runtime/Step
transactions keep their original2s budget; cancellation does not extend it.
Run requires no pending edit/import/gesture/draft/unsaved-decision or unverified
save receipt. Failure before first runtime publication retains original editing
presentation. Stop closes VM/tasks without callbacks, clears input/clock/queued
widget events and restores retained accepted draws/instances/store and selection
without compile/import/resource prepare. New Run starts at initialtick0 with a
new positive nonce and new VM. Old commands are rejected by monotonic play command
tokens in addition to Editor epoch/revision. No queued busy command executes after
Stop. Close first Stops, then invokes the original accepted-content Save/Discard/
Cancel policy; cancel leaves the original editable content.

Profile6 gets full parent-first local graph, local visibility and resource mapping.
Reset(run,actor2,actor3), identity/visibility/bind_resource/append_field/derive build
the initial world. action(space,x,y,toggle) snapshots input at actual dispatch;
tick(next,dt) receives positive integer sequence and fixed f32 dt1/60. Auto owns
translation/visibility changes and world/effective recomputation.2D speed60 units/s,
3D speed3 units/s, independent axes without diagonal normalization:2D left/right
worldX, up/down worldY;3D left/right worldX, up/down minus/plus worldZ. V nonrepeat
edges toggle the focused actor; even edge parity before dispatch cancels.

Each complete transaction queries profile/run/tick/count and every node, including
hidden nodes: exact ordinal/kind/dimension/parent/resource, local/effective boolean,
local translation, returned world and all static fields. At128 nodes this is2820
typed scalar results. Integer identities remain integers. Only local translation
and local visible may vary; count/identity/parent/resources/kind/material/yaw/pitch/
scale remain fixed. Local Document validation checks original numeric/dimension/
forest limits; CPU hierarchy is only the rejection oracle for returned world and
effective. Renderer/bounds/presented snapshots consume the actual admitted outputs.
Any wrong type/nonce/sequence/resource/static field/numeric/world/effective rejects
the entire frame, keeps accepted tick/pixels and closes the VM. No clamping, skipped
node, proxy model, Rust behavior fallback or VM rollback/continuation is claimed.

One inflight transaction, at most four unissued tick credits, at most four dispatches
per event-loop round. All callbacks/queries share one8ms round target, using inherited
1000-step/2ms slices,64-instruction polling,100000 steps/120 resumes/250ms callback
budgets. Init30s includes compile followed by a deadline check/config/output/prepare;
complete runtime tick callback+queries+prepare has2s deadline. Synchronous compile,
GPU/native calls and OS scheduling are not hard-preemptible sandbox operations.
Display gap>250ms or credits>4 auto-Pauses with reason, discards wall-time debt/input,
and completes only the already-issued transaction before Paused (or Faulted).
Every completed tick is consumed; display batching never drops accepted output.
The positive i32 tick limit faults before another dispatch instead of wrapping.

Pause rejects new ticks, clears input/debt and waits for a complete inflight boundary.
Paused stays unchanged; Resume establishes a fresh wall-clock baseline. Single Step
only in idle Paused sends exactly one neutral-input actual Auto tick, staying Paused;
busy/replayed step tokens do not queue another. Histories/frames are not accumulated:
one private candidate and one last-good frame, bounded query queue and4096 scalar
timing/action summaries.60Hz is logic dt, not a real-time promise for arbitrary loads.

Acceptance: authoring_play, authoring_play_host, deadline/completion tests,
check_scene_play.py and check_play_owners.py, plus source/consumer/affected old cases.

## SD-02 modify authoring-session/document/hierarchy/scene-assets

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

## SD-03 modify viewport-interaction

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

## SD-04 modify auto-source-driver/prototype-profile/project/integration-contract

Profile6 is a finite driver-local scene behavior sample alongside unchanged old
game and authoring profiles. Exact CLI: --play-check, --play-asset <GLB>,
--asset-save <scene>, --report <path>; negatives --wrong-play-focus,
--drop-play-tick, --play-gpu-fault transparent|missing-part|wrong-asset, --frames1.
Full positive reaches27 phases and six independent endpoint/origin/all-part pixel
states, actual Auto step+1, UI/V parity/hidden/shown/focus loss, Stop original canvas
hashes, fresh Run identity, Save and relocated independent-process reopen. Negative
exit/report/aggregate completeness agree; partial/absent oracle is failure. Real GPU
faults must fail the same pixel oracle despite valid protocol/state/mesh counters.

Source closure is plan839-identity.json and verify_plan839.py --controls; package
consumer exports only verified inputs plus pinned680 SDK files, with its own target.
Inherited worker stack16MiB and native PE32MiB settings remain required for a fresh
consumer. Existing model upload counters distinguish immutable geometry writes from
legal per-instance/content/composition writes. Report bounded Run/Stop/pump/full-frame
CPU/compile durations and Windows working set; exclude GPU timestamps, Iced internal
buffers/atlas and driver residency. No performance/production maturity is inferred.

No arbitrary scripts, dynamic graph/resource edits, persistence of runtime, Apply
runtime result, general TRS/ECS/physics/audio, SDK/AGP changes, production AutoUI/RQHost,
Forge distribution, ACE/AutoShader/CAP, Linux or new platforms. Source integration
remains autographics-dev only; master receives knowledge after the actual closeout.
