# PLAN-890 owned message closure

Includes the actual SDK message module as one source, without depending on
Core/VM/Axum/UI/Man. This is a source-level feasibility probe, not a published
contract crate. `a2r-std` supplies the existing version-independent upload
stream and file descriptor types; their identity is preserved.

```powershell
cargo test --manifest-path test/fixtures/modularity/web-owned-messages-probe/Cargo.toml --target-dir test/fixtures/modularity/web-port-probe/target -j 4 owned_messages_preserve -- --test-threads=1
cargo tree --manifest-path test/fixtures/modularity/web-owned-messages-probe/Cargo.toml --locked
```

The named test transfers actual requests and replies over OS threads. It
requires exact binary upload chunks, propagation of the source I/O error,
one terminal body drop, bounded SSE queue rejection, ordered SSE delivery and
EOF, and an unchanged file descriptor/request identity without file I/O.
Real VM/TCP and lifetime checks use `web-port-probe`; this case alone does not
establish VM owner scheduling or full resource isolation.
