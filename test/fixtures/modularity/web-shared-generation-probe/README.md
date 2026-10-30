# PLAN-890 shared generation feasibility

This isolated workspace compiles the actual `rust_workspace`, `sidecar`, and
`source_owner` source modules plus the restricted `shared_generation` language
mechanism. It deliberately has no mixed `api_gen` module,
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


The extracted language helpers have a real external compile/run control:

```powershell
cargo test --manifest-path test/fixtures/modularity/web-shared-generation-probe/Cargo.toml --target-dir test/fixtures/modularity/web-port-probe/target --locked -j 4 --test shared_companion -- --test-threads=1 --nocapture
```

It transpiles actual imported companion sources, uses the one public-type
promotion mechanism and compiles/runs through a temporary Cargo consumer of
Core. Removing promotion must give E0308 (`types::Work` / `works::Work`);
removing source_dir must give E0308 (`&str` / `String`). Restoring good generated
sources must run successfully again. The fixture reuses an explicit target for
this scoped check; independent fresh/locked/relocation delivery remains later.
The old mixed API public paths re-export the same implementation, not a copy.
