# PLAN-840 r8 final affected review

Verdict: **pass**, review_scope: final, for all work-owned r7/r8 acceptance and Spec changes. Reviewed ba6479da53b7999e490dd0e8ac8d701d5ffe86b2 against bfe7ed689 (executable baseline ede7f5a90); same clean lang-840/plan-840-dev and auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8, six frozen locks verified. Implementation writes paused. Same implementation conversation; no separate context/model independence claimed. Verdict reconstructed from committed diff, raw failures, mutation, real-system controls, actual configuration membership and review-scoped four-case replay, not executor checkbox totals.

| Acceptance | Verdict / evidence |
|---|---|
| AC01..10 original graphics/SDK support | retained pass: original reviewed1f1939525/delivery8b62db5c and source-only SDK932 pin unchanged; original63 invocation full frozen gates plus independent consumer/native evidence remain attached. New repairs neither remove API/source forms nor alter that frozen bundle |
| AC11..14 Phase2 feature/timing/selection | pass retained for unchanged feature/meter code and inputs; R7 exact config audit0→23 members and23/23 actual cases per full/t3, same existing serial group. No filters/features/layout oracles removed; whole-pool evidence deferred appropriately |
| AC15 affected review | pass current review; prior R6 review and canonical rules preserved |
| AC17..19 Phase3 Init/RQHost | retained pass: production UI helper and all test-only48+4 controls unchanged, commits already landed ede7; no D-762-1 production claim |
| AC21 lock production | pass: real Windows error5 reproducer rejects old immediate unlocked execution; bounded transient retry, unchanged2s/10s, only AlreadyExists stale branch, hard-error executes once, timeout/stale/100 updates/atomic adjacent controls pass. Last-error diagnostic retained; best-effort is explicitly not guaranteed mutual exclusion after deadline |
| AC22 SSE production | pass: first pull holds tasks map read guard and lazy startup inserts into same shard. Fixed collision red before repair; Arc clone drops registry guard before instruction execution. Nested driver shares invariant; isolated reversal reliably red, restored exact nested value/EOF/task cleanup pass. Original first/second/park/spawn<30/timeouts byte-exact, raw34 cases +200 repeats +4 review replay pass |
| AC23 health/spec/freeze | pass: committed clean tree, same code/config/locks after rebase, touched rustfmt/diff and locked check pass; SDK and stdlib canonical additions match actual bounded behavior. Existing compiler warnings are outside changed executable lines; no new diagnostic/debug workaround |
| AC16/20, T-10/T-16 | **pending merge-owned**: complete bare cargo t, VM tv and affected tt/tb on this freeze, then main single-instance full tf/tt/tb and receipt; no landing/archive/completion claim before gates |

Omission/deferral/workaround audit: no removed support, ignore, smaller input, weaker assertion, relaxed threshold, or invented pass. No new work-owned debt candidate; D-762-1 remains preexisting/outside this production scope. Original batch-r2 errno was omitted, so exact historic OS error remains unproven; the concrete Windows race is now tested and diagnostics improved, and required complete-pool replay remains pending. First POSIX-unlink fixture failed to create intended state, is retained and explicitly excluded as root-cause proof.

Spec-impact metadata includes SDK/project, auto-lang/project, stdlib/http-stream-lifecycle and existing Phase1/3 sources. Derived row/link changes add navigation only. No schema or documentation generator/syntax changes; docs_gen not triggered. No AAVM/UI-codegen/transpiler source touched; VM tv triggered, tt/tb are also retained as required closure legs because this Plan repairs those tested configurations/failures.

Next responsible role: merge. Full gates are pending; review pass alone does not authorize landing before them.
