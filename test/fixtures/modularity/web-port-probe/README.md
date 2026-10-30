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
