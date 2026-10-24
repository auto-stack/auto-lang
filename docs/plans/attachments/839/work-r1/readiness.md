# PLAN-839:r1 — work ready for independent review

**Work pass, execution_done 8/10.** Commit
`e58828ca64b7e26ce984faf850fb3b1d0887517f`, branch `plan-839-dev`, authoritative
tree `D:/autostack/.wt/lang-839/auto-lang`, base
`37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c`. Tree clean; implementation writes paused.
No independent final review, complete merge gate, source landing, canonical/ledger
publication, archival or cleanup occurred. Next responsible role: review, T-09.

[Structured receipt](receipt.json.txt) records complete identities, actual commands,
native exits/reasons, measurements, controls and pending gates. Source manifest has
278 inputs; [exact retained bytes](source-identity.json.txt), raw SHA256
`77fcb3c806b42a5caf7fa18326cdada15ae89db93464df6b5ea31537c0552765`, LF SHA256
`526f760d6a5e8eb74fd33bc0f3bd222fff76131f83b4da5b9cfc32469da7d0a3`.
Text transport is LF/CRLF equivalent under the verifier; receipt records raw and LF
Spec fingerprints too. Fixed SDK9576b4d/680 files,479-package closure/17 local/29
edges and both locks remain unchanged. Only driver paths changed. AGP/root production
Rust/dependency locks and canonical Specs/ledger are untouched.

## Acceptance evidence

| Case / AC | Actual path, assertions and current result |
|---|---|
| V-01 / AC01,07 | G838 archived10/10 and final delivered37fd1a9cf receipts/guard/cleanup/not-due checked before tree creation. prepare_sdk680 + metadata479/17/29; initial838 source264 and current839 source278 verified. Four actual source/resource/lock mutations fail; LF and restored controls pass. [Current source](producer/play-source-ready.log.txt), [controls](producer/play-controls-ready.log.txt). |
| V-02 / AC02 | `accepted_dirty_redo_unsaved_asset_isolation_stop_restore_files_and_reopen`: dirty plus redo and accepted unsaved two-part Model start from memory, complete Editor fingerprint unchanged. Draft/pending rejected in `draft_pending_actor_and_stale_operations_reject_before_side_effects`. [Host5](producer/play-host-readiness.log.txt); [new init recovery replay](producer/play-start-recovery.log.txt). |
| V-03 / AC03 | Actual profile6 full graph + typed integer header/metadata/local/world/effective/static fields. Twelve malformed output variants reject entire transaction, preserve last-good/tick and release actual VM. Changed Auto speed/visibility affects actual GPU endpoint/origin/hidden pixels in `changed_auto_speed_and_visibility_are_consumed_by_actual_gpu`; independent expected distinguishes default Rust movement. [Model6](producer/play-current.log.txt), Host5 above. |
| V-04 / AC04 | `retained_600_tick_trace_f32_reference_and_replay` compares every tick to an independent f32 reference and repeat run. `display_batches_dispatch_same_600_complete_ticks_without_coalescing` checks two display schedules against all600 complete results.128 nodes, zero-node world, numeric boundary, gap/backlog, type/budget, positive-i32 limit covered. [Model6](producer/play-current.log.txt), [actual deadlines](producer/play-frozen-deadlines.log.txt). |
| V-05 / AC04,06 | `pause_step_pending_stop_restart_gap_and_backlog` checks inflight pause, actual neutral Step+1, busy rejection, paused stability, Resume/new clock, pending Stop/owner release/new nonce. Host stale command/Step replay/restart checks; native Step verifies real returned tick+1 and subsequent stability. |
| V-06 / AC05 | Actual CompositionInput/window key route and real Iced path input focus:2D/3D arrows affect only their actor, UI keys do not move/toggle, two V edges cancel, single V hides actual top pixels, next V restores. Posted keys use actual scan codes/WM_CHAR; repeat/focus/axes/parity/clear siblings also have formal Input assertions. Native focus loss actually auto-Pauses, clears input, then Stop recovers. [Native positive/raw key events](producer/play-native-readiness/positive/report.json.txt). |
| V-07 / AC06 | Actual Session callback type/infinite-loop budget failure and start30s/full tick2s/post-prepare deadline close real VM owners. Host preparation failure keeps last-good canvas hash and tick; Stop succeeds with resource preparation deliberately still failing. Wrong initial profile leaves editing image unchanged, closes its actual weak VM owner, and the next legitimate Run succeeds at initialtick0. [Specific recovery](producer/play-start-recovery.log.txt). |
| V-08 / AC02,06 | Run/Paused/Faulted Save/Open/Import/New/Apply/Delete/Reparent/Undo/Redo/actor changes rejected; sentinel file bytes and directory unchanged, no deferred queue. Dirty Close Stops then Cancel preserves original accepted state. Save after Stop and new-process inspect restore only editable locals, history0/clean. Native source removal+project relocation+different-cwd fresh process matches saved Document/canvas hashes. |
| V-09 / AC07 | Actual20 cycles: runtime VM weak owner gone at fault/Stop; Editor source/mesh owners remain valid; GPU Arc, target build_seq and pipeline object identity remain; geometry upload18 writes/137208B unchanged while legal instance/content/composition writes grow. Editor close releases source, explicit external mesh pin retains until last drop. Actual retained-VM and forced-reupload mutations both exit101 at their required assertions, restored control0. [Owner controls](producer/play-owners-readiness/results.json.txt). |
| V-10 / AC05,06,08 |27-phase full native positive0/passed=true; six pixel stages with six independent checks each: editing/2d-moved/3d-moved/ui-isolated/shown/stopped. Wrong-focus, dropped-tick output, real transparent/missing-part/wrong-GPU-resource and frames1 incomplete all exit1/passed=false at named oracle. [Producer7](producer/play-native-readiness/results.json.txt), [independent consumer7](consumer/play-consumer-ready-native/results.json.txt). Existing window9/hierarchy16/assets20 positives all pass under actual widgets/scroll, original pixel assertions retained. [Old native reports](producer/play-frozen-old-native/window.json.txt), [hierarchy](producer/play-frozen-old-native/hierarchy.json.txt), [assets](producer/play-frozen-old-native/assets.json.txt). |
| V-11 / AC08 | README, actual CLI, `research/scene-play-feasibility.md`, `scene-play-spec-delta.md` and `scene-play-handoff.md` in frozen commit describe exact state/API, schema3/limits, included sample/no user-script product, measurement exclusions, source closure and unchanged full merge matrix. SD01..04 remain proposed, no live publication. I2/I3 remain pending. |

There are **36 distinct passing producer top-level permanent cases**,0 failed/0
ignored: model6 + Host5 + scratch1 + deadline1 + completion4 + affected old Host7/
interaction7/session4/scheduler1. Unrelated lib cases filtered out are not required
skips. Duplicate runs and expected-red controls are not added to passed-case totals.
[Affected19](producer/play-frozen-affected.log.txt), [completion4](producer/play-frozen-completion.log.txt),
[scratch1](producer/play-frozen-feasibility.log.txt). fmt/check pass with no authored
warnings; the pinned read-only SDK's146 inherited warnings remain explicitly known.

## Independent source consumer and evidence reuse

`D:/autostack/.wt/lang-839/consumer-work839` was exported source-only with its own
target and all680 SDK inputs. It verified source controls, all-target check, both
formal play targets11/11 and all seven native gates including relocated new-process
reopen. It uses explicit16MiB worker/32MiB PE stacks. No producer target/rlib/binary
was copied. [Current consumer11](consumer/play-consumer-ready-tests.log.txt),
[check](consumer/play-consumer-ready-check.log.txt), [source](consumer/play-consumer-ready-source.log.txt).

Last self-check correction clears Panel.error after successful play commands so
Resume cannot retain an old auto-Pause reason. Its new permanent assertion and the
strengthened failed-init weak-owner/next-Run assertion passed on producer and consumer.
Consumer refreshed exactly Host + play_host test + identity and rebuilt/replayed
check/play11/native7; SDK, locks, all other source bytes and its own target stayed
fixed. [Refresh identity proof](producer/play-consumer-refresh.json.txt). Model/clock/
deadline/completion/scratch and old authoring execution paths, dependencies, config
and fixtures were unchanged; prior frozen scoped tests and old native positives
remain applicable. No earlier result substitutes for the changed Host/native paths.

## Development failures and closure

No final-review rounds occurred. Failed development attempts are preserved, not
counted as passes. Initial native cyan expected linear rather than sRGB output;
oracle fixed to independent sRGB values. V injection needed real scan codes/WM_CHAR;
Resume and canvas clicks needed separate event rounds because a canvas correctly
clears prior Iced event batches. Synchronous GPU readbacks could trigger the actual
four-credit auto-Pause; probe now observes Pause, Resumes/refocuses via widgets and
reads hidden pixels while Paused. Production clock budgets were not relaxed.

Old fixed-coordinate list clicks were replaced by actual bounds without changing
selection/focus assertions. Old native clipped rows now use OS wheel messages before
their phase oracle. A bottom-edge direction error in that new probe navigation was
fixed. Event-only scroll offsets became stale when Delete shrank content; a read-only
public Scrollable operation now observes actual clamped translation before forwarding
the original operation unchanged. The assets phase20/9 pixel records and hierarchy16
original pixel controls now pass. `cargo fmt --all` encountered the pinned SDK nested
vendor workspace; correct inherited `cargo fmt -- --check` passes, SDK unmodified.
An authored duplicate DPI match warning was removed; current check has only the
known SDK warnings. No acceptance, fixture support, numeric limit or production
scope was weakened to close these issues.

Measured producer fixture: compile26.9ms, Run27.4–27.6ms, Stop0.73–0.82ms, maximum
runtime pump round2.2ms, frame CPU P50/P95 16.55/17.09ms, Windows working set371027968B.
Separate128-node actual init/tick/compile:33.32/18.74/34.77ms. These are observations
for those runs, not GPU timestamps, driver/Iced allocation accounting or a universal
60FPS guarantee. See structured receipt and raw records for exact integer values.

## Handoff

Work-owned AC01..08 pass. Open in-contract findings: **none**. Required work-owned
coverage gaps: **none**. Unresolved contract decisions/blockers: **none**. Proposed
SD01..04 are complete. This implementation-session self-check is not final independent
review. T-09/AC09 must independently verify this same committed tree and Plan r1;
implementation writes remain paused. T-10/AC10 and all complete frozen merge gates
remain pending: full locked driver/AGP tests/builds, original16 AGP cases, old games/
authoring native positive/negative/completeness, counter AND composed-content controls,
play/owner/source/fresh-consumer matrix, then actual autographics-dev landing, canonical
SD/ledger/archive, fresh guard/cleanup and batch-due receipt. Keep this tree and the
standalone source consumer. Master has no implementation changes.
