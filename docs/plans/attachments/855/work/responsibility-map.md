# PLAN-855 execution responsibility map (before implementation)

Baseline: 523d27b863dbe306c89af0ea546334225a29f8e9 (854 delivered); 853 delivered c1961665d. Both archived/delivered 9/9. SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467, unchanged locks/AGP/API/samples. Worktree D:/autostack/.wt/lang-855/auto-lang, branch plan-855-dev. User explicitly started work on 2026-10-27.

Choose Host child-module impls over private adapter objects: no new storage, owner, callback cache, event queue or public mutable field. Root retains Host/Panel/Prepared types and constructor. Existing private methods crossing siblings become pub(super) only. Existing public methods remain at authoring::host::Host. Play keeps its existing single struct and scheduler; splitting its state machine creates no benefit for this boundary.

| Module | Actual path / responsibility | Ownership, failure and closure |
|---|---|---|
| host/panel.rs | Panel sync/drafts + SurfaceUi view/update | Panel retains UI-local drafts/bounds; stamped commands still enter Host::dispatch, no direct Editor publication |
| host/commands.rs | dispatch, request_leave/perform_leave, open/settle | Editor retains accepted/pending/history/code/assets; Storage retains receipt/root. Candidate + prepare precedes one accepted/history publish. Busy/stale/cancel never replay queued edits |
| host/play_adapter.rs | source selection -> start_program -> Play::new_user -> pump_play -> prepare_frame/publish; stop/pause/token dispatch | Host owns one Option<Play>, Play owns one VM/candidate and clocks; retained Prepared restores editor without rebuild. Failure keeps last complete frame, Stop drops runtime owner. Starting cancellation remains in Play |
| host/viewport.rs | prepare/publish + frame + pointer/key/blur/resize/camera/gesture + geometry/scroll | Same Host input/runtime/presented snapshot and capture token; real Auto Building produces candidate, freshness checked before publish. Esc/focus/DPI/resize/Play cancel unchanged. No duplicate input state |
| root host.rs | structs, initialization, UI sync, active_assets/prune/model and resource/probe introspection | Editor/history/Play and permitted external pins retain immutable resources. Prune and actual last Weak/Drop are lifetime oracle, counters alone insufficient |
| bin/authoring/cli.rs | original main parser, report then exit | Exact parameters/defaults/error propagation; ordinary lifecycle stays root App |
| bin/authoring/probes.rs | finalize_checks, legacy_scroll/probe_step | Same App probe fields; normal runtime has no probe-driven advancement unless flags select it; partial/error fails before report |

## Execution contract / targeted closure

V-01: unchanged authoring_session/storage and behavior_project_session/storage assertions inspect accepted/hash/dirty/history and disk reopen; valid edits vs stale/noop/failure controls distinguish accidental transaction changes.
V-02: authoring_interaction/host and existing native interaction probes inspect target/capture/actual Auto publication and canceled late results. Moving methods does not grant direct state access to probes.
V-03: authoring_play_host/behavior_host/motion_host inspect actual VM, full config, Pause/neutral Step/Resume, last-good and Stop/new Run. No new scheduling implementation.
V-04: motion_owners and existing host lifetime cases inspect Weak/actual last-owner release, legitimate external pin and fresh-upload negatives. Required native complete matrix remains merge-owned.
V-05: original bin completion tests + authoring_probe_exit/motion_cli exercise partial/failure/control and report exits. Tests retain original oracles.
V-06: new current closure verifier must reject removed moved modules/unlisted source/wrong SDK then accept restoration; own-SDK consumer actually runs. Frozen 853/854 ten fixtures and 21 generated files compare byte-for-byte. Same five workloads, off/on, five new processes each and 30 warm ticks compare semantics/resources and report times honestly.

All work checks scoped. Frozen producer/consumer complete/native-owner-content gates are merge-owned after independent review. No root Cargo tiers. Existing SDK146 warnings recorded, driver must add zero. No unresolved semantic decision; module placement is equivalent local implementation within r1.
