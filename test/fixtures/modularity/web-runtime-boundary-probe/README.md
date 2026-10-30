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
