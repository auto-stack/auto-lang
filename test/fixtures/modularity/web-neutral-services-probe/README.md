# PLAN-890 neutral photo service feasibility

The probe includes the unchanged photo service source from the pinned sibling
`auto-ui` worktree. It depends only on image codecs, EXIF and blake3, matching
the existing media foundation's dependency families. It has no UI runtime or
renderer dependency. The sibling is read-only; no production ownership has
moved yet. This fixture requires the PLAN-890 sibling layout.

Run the named photo module tests serially with `TEMP` and `TMP` set **only in
the test child** to a private directory. The thumbnail cache uses `temp_dir()`;
isolation avoids touching a shared cache. Restore the parent environment after
the child exits. The work receipt records the actual command, source hash,
private directory and output hash. Do not use the default global cache for this
probe.

```powershell
cargo test --manifest-path test/fixtures/modularity/web-neutral-services-probe/Cargo.toml --target-dir test/fixtures/modularity/web-port-probe/target -j 4 photo_service::tests:: -- --test-threads=1
```

This proves the existing photo implementation can compile outside UI. Moving
its single source into `auto-media-core` and changing consumers still requires
the Plan's later migration steps; these ten tests do not close those steps.
