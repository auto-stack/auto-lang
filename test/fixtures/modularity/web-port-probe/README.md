# PLAN-890 Core-only HTTP baseline

This bounded probe records existing public SDK behavior before deciding the
HTTP host port. It is not the complete Web port, a migration fixture, a
replacement for existing HTTP tests, or an acceptance pass for PLAN-890.

From the PLAN-890 language worktree:

```powershell
cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --locked -j 4
python test/fixtures/modularity/web-port-probe/probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe
```

For the first build, omit `--locked` to resolve the fixture's local lock.
The package depends only on `auto-lang-core`. It calls `run_with_capture`
without registering an HTTP provider. The controller requires HTTP 200 with
the exact per-invocation token, sends the existing test-support shutdown
request and requires the child to exit successfully. Build failure, early
exit, wrong/missing body, undelivered shutdown and failure to exit are red.
An unrelated listener cannot pass the exact-body readiness assertion.

The controller owns only its child process and temporary source/log directory.
If the probe fails, it reports the failure before reaping that child. It does
not kill a port owner or claim a forced kill is graceful shutdown. The
temporary logs are printed before deletion; callers should capture output to
their local evidence directory.

The test proves the JSON/default-assembly baseline on the executed platform.
It does not establish candidate host assembly, streamed uploads, SSE, file
lifetimes, a new package DAG, Web distribution or four-repository delivery.
