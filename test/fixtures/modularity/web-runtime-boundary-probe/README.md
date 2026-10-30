# PLAN-890 external runtime boundary

This package compiles the actual service source by path in an external crate.
It imports the existing public Core SDK and shared owned message types; no VM,
AST, Store or framework source is copied into the fixture. The required
`boundary` feature enables the audit. An empty default build is not evidence.

```powershell
cargo check --manifest-path test/fixtures/modularity/web-runtime-boundary-probe/Cargo.toml --features boundary --lib --target-dir test/fixtures/modularity/web-port-probe/target -j 4
cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --features external-runtime --locked -j 4
python test/fixtures/modularity/web-port-probe/response_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe --expect-external
python test/fixtures/modularity/web-port-probe/upload_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe --expect-external --cancellation
python test/fixtures/modularity/web-port-probe/isolation_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/isolation.exe --expect-external --admission
```

For the initial fixture lock resolution, omit `--locked`. All external-mode
commands require explicit markers from the compiled external branch so a stale
Core-runtime executable cannot certify this boundary.

`serve_owned` consumes `HttpVmOwner::into_local_parts`, then runs this crate's
transport/dispatch/scope implementation using the public VM object, immutable
compiler metadata, eleven named VM resource/async bridge operations, and a
host-owned request context. It does not call Core's temporary runtime-start
bridge. VM and context-restoration guards remain !Send. The payload of the
request context may be shared with workers for deadline/resource reporting.

The callback closes native resources in the serving host's actual scope, even
though their language registries remain in Core. Runtime checks inspect this
crate's scope table and the real Core resource registries. Upload cancellation
requires observing nonempty staging before RST, followed by empty staging and
no published partial file. It cannot pass by never starting receive.

This is a bounded source-boundary probe, not production Web delivery. Core
still compiles the original framework and the file/upload executor remains at
its original owner until T-05. Configuration/observer and direct-listen paths,
required test migration/dev closure, final package graph and distribution
remain Plan work. The original inline test inventories are not run here and
the source inclusion is not a substitute for their T-08 migration.

Configured-instance control and observation have a separate real-TCP oracle:

```powershell
python test/fixtures/modularity/web-port-probe/service_state_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/isolation.exe
```

It requires distinct identities and exact bound addresses/config hashes,
unchanged B counters after A traffic, per-origin CORS with no legacy wildcard
leak, and headers-only shutdown of A while B remains ready. Both owner futures
join and the actual resource tables must be empty. The optional
`--shared-observation-negative` mutates only this fixture's runtime factory to
share the old process observation; it must exit nonzero for shared identity
while still draining both services. This is an oracle control, never a product
configuration or a green delivery result.

Network-thread termination can be distinguished from listener closure by setting
the **fixture-only** `PLAN890_NETWORK_TAIL_MS=800` when running the installed
`web-port-probe/probe.py` command. After executing the entire original transport,
the wrapper keeps that same real network thread alive for 800 ms. The single
owner's terminal assertion rejects returning with a live tail; the original
owner loop fails it. There is no production delay setting or alternate transport.

The configured state probe accepts `--idle-on-shutdown` to retain a partial-head
connection through the drain timeout. Its adapter checks the actual instance's
active-connection counter after the serving future returns. Restoring the
original transport while keeping the new owner join makes this oracle fail:
the old async cleanup was skipped when the connection future was canceled.
`--file-during-drain` receives an exact 16 MiB deterministic file across shutdown
and requires readiness to stop returning 200, then verifies independent B ready
and port closure. These are separate checks and keep their source/assets under
the worktree fixture target. `probe.py --mode pre-stopped` also exercises an
already-true watch before activation and requires successful return/rebinding.
