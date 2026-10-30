# PLAN-890 generated service neutral closure

This external consumer invokes the real production API generator. The Python
oracle creates a new project, generates its server without a UI source root,
compiles the generated Rust unchanged, repeats the locked build, and verifies
the complete Cargo package graph excludes the UI, Man, Tauri and unused language
facade. It reuses an explicit target for this scoped feasibility check.

```powershell
cargo build --manifest-path test/fixtures/modularity/web-generated-service-probe/Cargo.toml --target-dir test/fixtures/modularity/web-port-probe/target --locked -j 4
python test/fixtures/modularity/web-generated-service-probe/probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/web-generated-service-probe.exe --target test/fixtures/modularity/web-port-probe/target --evidence test/fixtures/modularity/web-generated-service-probe/target/runs
```

The process must become ready with the exact generation assembly fingerprint,
return this invocation's business value, scan a 3x2 PNG, serve identical original
bytes and a real decoded JPEG thumbnail with matching dimensions, reject path
traversal/unknown tokens, and stop through its own control endpoint. The ready
and stopped receipts are read from stdout, matching the production protocol.
Afterwards changing the API source must invalidate freshness and restoring the
same bytes must restore it. Raw generated artifacts, logs and JSON stay local in the worktree; the oracle
rejects a scratch location outside that tree.
This is not the final independent Web package, clean-target SDK distribution,
four-repository matrix or UI application acceptance.
