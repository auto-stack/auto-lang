# PLAN-890 shared generation feasibility

This isolated workspace compiles the actual `rust_workspace`, `sidecar`, and
`source_owner` source modules. It deliberately has no mixed `api_gen` module,
UI crate, Man crate, or Tauri dependency. No workspace or sidecar algorithms
are copied here. This is a bounded dependency proof, not Web delivery.

```powershell
cargo test --manifest-path test/fixtures/modularity/web-shared-generation-probe/Cargo.toml --target-dir test/fixtures/modularity/web-port-probe/target -j 4 workspace -- --test-threads=1
cargo tree --manifest-path test/fixtures/modularity/web-shared-generation-probe/Cargo.toml --locked
```

The service workspace test runs in a child process with private source roots.
It requires successful service-only generation without UI dependencies, rejects
the missing explicit UI source through the existing UI entry, and rejects a
service rewrite when an existing target-specific renamed dependency needs UI.
Both failures must preserve the previous manifest bytes. The existing source
receipt and sidecar tests remain available; they are not execution credit for
the Plan's full migration inventory.
