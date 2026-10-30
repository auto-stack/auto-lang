# PLAN-890 HTTP host feasibility probes

These bounded probes check the explicit host handoff. The original Core-only
implicit-default baseline is preserved at commit `6d70b2267` and in the Plan's
historical evidence. This is not the complete Web port, a replacement for
existing HTTP tests, or a final acceptance pass for PLAN-890.

From the PLAN-890 language worktree:

```powershell
cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --locked -j 4
python test/fixtures/modularity/web-port-probe/probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe
```

For the first build, omit `--locked` to resolve the fixture's local lock.
The package depends only on `auto-lang-core`. In the default `installed` mode,
the driver explicitly installs a fixture adapter implementing the public host
trait, then calls `run_with_capture`. The controller requires HTTP 200 with
the exact per-invocation token, actual test-support shutdown and successful
process exit. An unrelated listener cannot pass the exact-body readiness
assertion. The fixture adapter still uses a temporary Core runtime bridge;
that bridge must be removed when the implementation moves into Web.

Two independent-process controls use the same SDK entry without installation:

```powershell
python test/fixtures/modularity/web-port-probe/probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe --mode missing
python test/fixtures/modularity/web-port-probe/probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe --mode plain
```

`missing` requires a nonzero SDK exit and the explicit missing-adapter diagnostic,
with no listener/adapter activation. `plain` requires result 42 and successful
exit, with no HTTP host requirement. Neither mode may be replaced by a
permissive implicit runtime fallback.

The controller owns only its child process and temporary source/log directory.
If the probe fails, it reports the failure before reaping that child. It does
not kill a port owner or claim a forced kill is graceful shutdown. The
temporary logs are printed before deletion; callers should capture output to
their local evidence directory.

These tests establish a prototype host handoff on the executed platform. They
do not establish complete per-instance resource isolation, a new package DAG,
Web distribution or four-repository delivery.

The owned-upload seam has an additional bounded real-TCP replay:

```powershell
python test/fixtures/modularity/web-port-probe/upload_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe
```

It reuses the actual PLAN-730 upload handler program, preserves the 12 MiB
accepted multipart input and checks exact download bytes, occupied-target
preservation, business rejection/staging cleanup and headers-only early
authorization rejection. A fixture-only random marker verifies listener
identity before uploads. It is a scoped seam replay, not execution credit
for the whole original suite or proof that explicit Web host assembly exists.

`compile-fail-owner-send` is an opt-in compiler negative control: checking the
fixture with that feature must reject `HttpVmOwner: Send` with E0277 because
the VM owner contains `Rc<AutoVM>`. Leave the feature off for ordinary builds.

The owned reply seam also replays the original file-handler program and SSE
generator over real TCP:

```powershell
python test/fixtures/modularity/web-port-probe/response_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe
```

It requires exact sync/async binary files, HEAD metadata, Range, ETag, missing
file and invalid options, ordered SSE frames and EOF. After successful owner
return, the driver now checks real scope/file/upload/stream/live-op registries
are all zero. The upload replay includes headers-only early rejection and a
path-argument binding failure after capability injection. These checks exposed
and now guard a leaked unconsumed upload capability; successful HTTP status or
process exit alone was insufficient. They remain scoped prototype evidence,
not execution credit for the entire historical test families.

Two live services can also be checked on the same local VM owner thread:

```powershell
cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --bin isolation --locked -j 4
python test/fixtures/modularity/web-port-probe/isolation_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/isolation.exe
```

The programs deliberately reuse handler names with different parameter and
return types. Before compiling the second program, the first must serve exact
JSON, binary file, SSE frames and headers-only upload rejection. The same
checks must remain valid afterwards. The first is then shut down independently
while the second must still answer; both owner futures are joined and real
resource tables must be empty. The pre-repair JSON failure is retained as local
evidence. This uses a temporary injectable-shutdown entry to the single existing
runtime, not a second framework implementation or the final Web SDK port.

Add `--admission` to the Python isolation command to set a one-request budget
in that child only. A 64 MiB streamed file and small receive buffer hold the
first service's response body open: its next request must get 503 while the
second service still returns its exact value. Resetting that one owned socket
must reclaim the first permit, restore its 200 response, and leave all real
registries empty after independent shutdown. The file is local temporary input,
written in 1 MiB blocks and never added to Git. This is a body lifetime/instance
isolation check, not a throughput benchmark.

For the actual external-crate runtime handoff, use the `external-runtime`
feature and commands in [the boundary probe](../web-runtime-boundary-probe/README.md).
`--expect-external` refuses a stale Core-runtime binary. The upload probe's
optional `--cancellation` case observes actual receive/staging, resets that
connection, and requires cleanup without publishing a partial file. In the
two-owner driver, repeat installation preserves the first provider, explicit
replacement starts the second, and clearing the default afterwards must leave
both active services usable. Local ownership compile negatives reject both
the VM owner and the thread-local context restoration guard as Send.


Borrowed synchronous listeners can be checked independently:

```powershell
cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --features external-runtime --bin blocking --locked -j 4
python test/fixtures/modularity/web-port-probe/blocking_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/blocking.exe --mode installed
```

`installed` runs a real annotated handler via the borrowed explicit host and
external single-source loop. It asserts exact JSON, missing/invalid arguments,
404, controlled return, unchanged task count and same owner thread. Additional
modes are `direct-installed` (actual native called inside Tokio),
`direct-missing`, `unsupported` (async-only custom provider) and `missing`
(annotated activation). Negative modes require explicit diagnostics and verify
the requested port remains bindable. The controller wakes accept using its own
socket; it does not certify slow-body cancellation. Language source method
resolution, legacy stdnet behavior and post-main coordination remain separate
T-02 obligations. Full scope and local evidence hashes are in
`docs/plans/attachments/890/work/blocking-listener-port-r2.md`.


The actual VM language direct entry is also covered using
`http.server_listen(http.server(), address)` after `use auto.http`. Add mode
`language-installed`, `language-missing` or `language-unsupported` to the blocking
probe command. Installed mode executes the full source/handler/JSON/argument
rejection path and must return after exactly one explicit listener; it forbids
post-main async activation and joins its owned shutdown controller. The two
negative modes require the error at native execution (RuntimeError), before a
socket is bound. Method-style `server.listen(...)` remains a pre-existing
unresolved spelling and is not advertised as a newly working language form.


The distinct old stdnet SDK grade has its own actual wire oracle:

```powershell
python test/fixtures/modularity/web-port-probe/legacy_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/blocking.exe --mode installed
```

It calls the real public compatibility SDK entry, which selects the explicit
host with `BlockingListenerKind::LegacyStdnet`. It checks wildcard CORS,
header-only OPTIONS, exact JSON and argument-binding failures, original finite generator
SSE frames (including a legacy int-declared handler returning the real generator), repeated explicit file-unsupported 500, and recovery after resetting
a live infinite generator. A separate caller-owned real lazy generator occupies
iterator key zero; ordinary integer zero must remain exact JSON and leave that
generator untouched. Request tasks/iterators/descriptors and retained credential
ownership are checked before VM destruction, separately from final drop counts.
`missing` and `unsupported` preserve the old unit-return surface while requiring
explicit diagnostics and no listener. `grade-swap` deliberately substitutes the
other runtime in the fixture: the wire oracle must reject it (exit 1), so exact
JSON alone cannot certify the legacy grade. The synchronous grade makes no new
slow-body timeout/deployment-policy promise.

Native policy handoff has a separate two-owner check:

```powershell
python test/fixtures/modularity/web-port-probe/policy_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/isolation.exe --plain-exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe
```

It uses the real `http.rate_limit` native before service startup and inside a
handler. The legacy SDK process default is deliberately one request; VM A
overrides it with four, and VM B explicitly disables it before dynamically
enabling two. Exhausting A must leave B unaffected, and changing B must leave A
exhausted. Exact nonce responses, 429 body/deadline/Retry-After/request-id,
legacy environment CORS, resource termination and two drained owners are required.
The same probe also runs a config-only ordinary program without an adapter.
Its source/build scratch stays inside this worktree's fixture target directory.

Compile-to-activation isolation is checked separately:

```powershell
python test/fixtures/modularity/web-port-probe/metadata_gap_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/isolation.exe
python test/fixtures/modularity/web-port-probe/metadata_gap_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/isolation.exe --ordinary-first
```

A's actual main parks on `delay_async(...).await`; B then compiles and activates
on the same owner thread before A activates. Same handler names have different
parameter/return types, routes and async declarations. Exact invocation values,
the actual external framework's async metadata reader, valid and invalid binding,
and both resource terminals are checked. Ordinary-first mode clears the default
adapter before A resumes: A must return 42, with no listener or provider call,
while B keeps serving. The fixture's stdin controller is joined after shutdown.

The normal compiler paths freeze a `CompilationMetadataGuard` onto the VM before
main or module initialization can await. Custom SDK compilers can use the same
guard and `VmHttpBridge::bind_program_metadata(&mut vm, guard.finish(&routes))`
before sharing/running their VM. Only owned declarations cross the boundary;
the guard remains local. Legacy standalone record/read APIs retain their cache
fallback, which is not a concurrency contract for custom unbound raw VMs.
