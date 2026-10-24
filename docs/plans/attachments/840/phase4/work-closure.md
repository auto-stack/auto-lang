# PLAN-840 r8 work closure

Frozen ba6479da53b7999e490dd0e8ac8d701d5ffe86b2; same lang-840/plan-840-dev; six dependency locks exact, auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8. Main contract bfe7ed689. No new Plan, agent or checkout. Rebase preserves all executable sources/config; only main contract/navigation placement differs.

| Finding / invariant | Evidence | Result |
|---|---|---|
| R7 full/t3 shared FFI cache isolation | phase2/r7 group-audit and actual family logs | each existing group changed from 0 members to all23; each23/23 pass; no test/feature/input removed |
| Windows lock namespace race | phase4-lock-negative-before.txt | actual kernel legacy disposition gives ACCESS_DENIED5; old implementation executes unlocked immediately; original batch lacked errno, not claimed identical in every historical occurrence |
| Bounded retry and legitimate downgrade | scoped-final.txt; stress-final.txt | delete-pending persists through25ms -> Degraded once; release -> Executed; hard error, fresh contention timeout, stale takeover, counter100 and atomic writes pass |
| SSE first pull registry re-entry | phase4-stress-before.txt; phase4-negative-before.txt | original random test39 passes then first-pull120s timeout; fixed shard collision reproduces5s outer diagnostic timeout |
| Nested generator registry re-entry | nested-mutation.txt | only restoring old inner guard yields deterministic5s timeout; restored fix yields nested exact value, EOF and empty task registry |
| Existing streaming semantics | scoped-final.txt |34/34 actual tests: original exact first/second/spawn/time assertions, queue bounds, cancel/permits/cleanup, UTF8/bridge and ordinary generators |
| Stability | stress-final.txt |50/50 iterations,4 distinct cases each =200 passes; this is scoped evidence, not substitute for batch |
| Health | check.txt;health.json.txt;locks.json.txt |locked cargo check, touched format/diff, no temporary diagnostic prints; existing unrelated warnings retained, unused Mutex removed |

SD-P4-01/02 prepared in canonical SDK/project and stdlib/http-stream-lifecycle, derived Plan navigation updated. Owned production scope is the two authorized failure paths; D-762-1 production UI helper untouched. Original graphics/source pin/support and every original acceptance requirement retained. All original SSE function text remains exact after removing only deterministic setup/helper/new sibling test (freeze.json.txt). No lowered timeout, spawn bound, wall threshold, assertion, feature or selection.

Work-owned tasks done; next role review. Complete t/tv/tt/tb and main-only batch remain merge-owned pending.
