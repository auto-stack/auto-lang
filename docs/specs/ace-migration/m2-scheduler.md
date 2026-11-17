# PLAN-968 — M2-A Task Actor & Event Scheduler (`av.scheduler` lane)

Status: delivered (M2-A slice; PLAN-968, reviewed commit aa630a2eb).
Scope: `task`/`on` actor declarations, `Task.spawn`, handle `send`/`ask`/`stop`,
`reply`, `await`, `go`, `sleep`, bounded mailboxes, virtual timers, and the
orderly stop-hook drain — with the same observable semantics on the AV lane
(deterministic cooperative scheduler) and the a2r lane (Rust emit: std threads
+ bounded channels). Source front door: `auto.source.scheduler.draft`
(aggregates superset + the task actor face + the legacy `{ident}` f-string
dialect). The **native (AC) lane executes the same surface as real machine code**
since PLAN-1067 (`mn.sched.runtime/1`; see
[native-async-scheduler.md](../auto-ac/native-async-scheduler.md)); the A2C lane
remains outside its original support domain. Three implementation-consistency
rules are shared by all three executing lanes: `Task.spawn` only in the program
entry (`main`), mailbox `cap >= 1`, and `stopping()` reports only on an empty
mailbox (buffered deliveries always precede the stop hook).

## Verified support (producer → consumer)

| End | Status | Evidence entrypoints |
|---|---|---|
| AV (`ac-av/scheduler-source-run-v1`) | Full | `ace-av` conformance `sched_m2_968` (17 cases, 0.07s); case-runner corpus `case-ace-m2-scheduler-968` (11 cases: canonicals `message_handler.at`/`spawn_with_state.at` + FIFO/ask/go/backpressure/stop-hook/virtual-timer + 3 named negatives) |
| a2r (Rust emit) | Full | `ace-a2x-worker` conformance `sched_968_a2r_rust_emit` (8 cases incl. `spawn_with_state`, byte-exact stdout) + `sched_1067_a2r_timer_and_closed_task` / `sched_1067_a2r_capacity_and_consumed_future_are_named` (real-timer, closed-task, cap<1 and consumed-future named outcomes) |
| Native (AC) | **Full** (PLAN-1067) | `ace-native` conformance `sched_m2_1067` (28 cases: the same 8-case corpus byte-exact as real PEs + closed-future/cancel/resource/import-audit cases); contract + evidence in [native-async-scheduler.md](../auto-ac/native-async-scheduler.md) |
| A2C | Outside original support domain | A2C complete alignment is an enhancement item (M2-B); the lane is not a parity gate |

## Semantic contract (frozen in PLAN-968 §5.1, D1/D2)

- `task Name { <state fields> fn start() ! {} fn stop() ! {} on { arms } }`;
  state fields default from int/bool/Str literals; `Task.spawn(name, cap, inits..)`
  initializes fields positionally. Desugar (`sched_desugar`) lowers each task to
  one driver function `__task_<Name>(inits...)` with the recv/dispatch loop.
- Handles/futures are monomorphic per task (`VT::Task(i)`/`VT::Future(i)`);
  at runtime both are opaque i32 ids. Mailbox capacity = the spawn argument;
  a full mailbox parks the SENDER (bounded backpressure, FIFO delivery).
- `ask` opens a 1-based reply port; `reply(v)` publishes on the current
  message's port (send-mode reply = named run-stage error); `await` parks the
  caller until publication (ports are single-consumer).
- `sleep(ms)` parks on a virtual clock (AV) / real thread sleep (a2r);
  program exit runs the orderly drain: mailboxes close (buffered messages
  still deliver), stop flags wake drivers, stop hooks run, all threads join.
- No late commits: send/ask on a stopped or closed task is a named error.
- Text crossing a scheduler handoff materializes into an owned buffer
  (AV `sched_materialize`; a2r owned promotion at field-let).

## MIR/contract seams

- MIR opcodes `SCHED_*` (0x41–0x4B) ride the closures wire family additively
  (the PLAN-965 LANG_FAIL precedent): encode/decode, mir_verify def/use +
  type rules, ownership arms, native named rejection. Console host imports
  are admitted for scheduler modules (`auto.console.print.*` → HostCall).
- Per-task intrinsic symbols (`__sched_spawn_<task>` …) carry the resolved
  fn index + exact signature; lowering rewrites call sites to the SCHED ops.
- AV execution: `execute_sched` multiplexes chains round-robin with park
  points (mailbox empty/full, pending reply port, sleep deadline); parking
  instructions rewind ip and re-execute on wake (idempotent ops).

## Deferred/out-of-scope

- B-phase consumers and deep incremental editing are not part of the M2-A
  slice. Streams ride the 965 generator ownership rules; no new platform
  promises beyond the frozen D1–D6 table.
