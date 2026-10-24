# PLAN-840 proposed Spec delta (r2)

Proposal only. Canonical Specs and the derived ledger are merge-owned.

| Delta | Proposed contract | Implementation / evidence |
|---|---|---|
| SD-01 | Versioned local user source, exact snapshots and callback admission alongside Builtin6 | behavior.rs, authoring_behavior; profile7 never silently falls back |
| SD-02 | Public Auto behavior/1 API: ordinal handles, local/world/effective reads, local translation and local visibility writes, scoped input | api/behavior_v1.at and api/README.md are the signature/semantics source; actual Auto callbacks and GPU tests |
| SD-03 | Source selection only in Editing; scene-root relative paths, independent scene/script persistence; app_start once after configuration and app_update once per accepted tick | host.rs, play.rs, portable Move/Patrol, host isolation and cancellation tests |
| SD-04 | Whole-frame validation, sticky first API error, bounded source/output/diagnostics, resource reuse and ownership, native positive/negative/partial evidence | existing frame checks plus profile7 header; source/consumer/owner/native tests; all old profile gates retained |
| SD-05 | Dependency source scanning ignores comments and quoted strings, including multiline content, while real use/dep syntax and traversal semantics remain | canonical dependency_scan/use_scanner/dep_scanner, 38 scanner tests and actual SDK Session/module cache controls |

Public source is one regular `.at` file, strict UTF-8 without BOM, at most256KiB.
Managed units are at most64KiB and the complete unit at most320KiB. Runtime
supports128 ordered nodes. Handles belong only to one run: zero is valid,
negative one means absent actor/parent. Invalid write/query arguments set the
first error, reject that frame, close the VM and preserve last-good state.
User source is snapshotted with raw/API/bridge/combined hashes, source regions,
resolved path and base scene/assets/actors/stamp. SDK byte spans are mapped to
original source; absent spans are explicitly reported, never invented.

No scene schema or package format change, external dependency loading, hot reload,
per-object script persistence, rotation/scale writer, native allocations hard quota
or sandbox guarantee is introduced. Ordinary Core helpers/state/control flow and
inert comments/strings remain supported. Bounds are source/host queue/frame/time
bounds, not a general VM heap accounting claim. See API documentation for exact
signature and errors. Both existing and new merge-owned matrices must pass on
the reviewed frozen revision before depositing these changes.
