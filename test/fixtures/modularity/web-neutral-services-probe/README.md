# PLAN-890 neutral photo service consumer

The probe now consumes `auto-media-core::photo_service` directly. The unchanged
implementation and ten original tests have moved once to that neutral owner;
the pinned UI tree keeps its existing public path as a re-export. No sibling
UI source is needed to compile this fixture. Its library has no local tests:
`cargo test` selecting zero cases is not a photo verification receipt.

```powershell
cargo check --manifest-path test/fixtures/modularity/web-neutral-services-probe/Cargo.toml --target-dir test/fixtures/modularity/web-port-probe/target --locked -j 4
cargo test -p auto-media-core photo_service::tests -j 4 -- --test-threads=1
```

Run the named photo module tests serially with TEMP and TMP set only in that
child to a private directory. The thumbnail cache uses `temp_dir()`; isolation
avoids using or modifying a shared cache. The Plan receipt binds the actual
command, original source hash, private directory and log hash. Production
HTTP/photo consumption is additionally checked by the generated service probe;
fresh SDK/distribution and the complete Web migration remain separate tasks.
