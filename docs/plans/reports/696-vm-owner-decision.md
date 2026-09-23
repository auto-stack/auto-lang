# PLAN-696 VM owner spike decision

## Decision

Keep the VM and its HTTP service on the execution thread and drive both from a
Tokio `LocalSet`. Synchronous execution entry points now enter through
`block_on_autovm_local`; `execute_autovm_with_path` starts the async server on
that same thread. `serve_async` accepts `Rc<AutoVM>` and each accepted
connection's `spawn_local` future owns an `Rc` clone. No `AutoVM` pointer is
encoded as an integer or reconstructed with `unsafe` in the async HTTP path.

This fits the existing execution model: `AutoVM` is `!Send`, and the connection
handlers already perform synchronous VM operations. An OS-thread handoff would
either violate ownership or require a larger request/owner actor bridge, which
is reserved for the later transport design.

## Lifetime and close behavior

The execution future owns the VM until it enters `serve_async`. The service
future then owns an `Rc`, and every accepted connection owns another clone for
the lifetime of its local task. A completed or disconnected connection drops
its task and clone. The server is currently process-lifetime and has no
graceful-shutdown API; if the listener loop ends, the `LocalSet` drops its
remaining local tasks and their handles. SSE cancellation is handled in T-06,
where the frame producer will also stop after a failed socket write.

## Evidence

- Base revision: `051e7b54023f420c3955c480d986e83197eb5899`.
- `cargo check -p auto-lang`: passed on the Plan 696 worktree.
- Owner smoke: `cargo nextest run -p auto-lang --lib --features test-http-e2e -E 'test(/e2e_plan696_body_split_across_tcp_writes|e2e_sse_generator_handler/)' --no-fail-fast` — 2 passed. The test harness invokes the same synchronous AutoVM entry point, which now enters a `LocalSet` before serving.
- `AutoVM` values are held through typed `Rc` ownership in `serve_async` and connection tasks. The HTTP server's previous raw-pointer-to-`usize` conversion and the SSE pull thread's equivalent conversion have been removed.

## T-06 completion

SSE frame pulls now run in 4,096-instruction batches and yield to the LocalSet
between batches. `Time.sleep_ms` inside an SSE generator records a wake deadline
instead of blocking the LocalSet; ordinary VM sleep behavior is unchanged. A
bounded channel carries frames to the socket writer, a heartbeat lets the
connection detect a closed peer during a sleeping generator, and dropping the
producer removes its iterator and VM generator task.

Evidence: `e2e_plan696_slow_sse_does_not_block_health` passes with a 2.5-second
generator delay while the concurrent health response remains under the 1.5-
second bound. `e2e_plan696_sse_disconnect_cancels_generator` passes and confirms
the post-sleep side effect does not run after the peer closes.
