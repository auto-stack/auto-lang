# PLAN-825 same-source module harness

This independent workspace compiles `../../core/src/loaded_program.rs` directly
against the delivered core's public APIs. It does not register a production
module, implement an executor, construct a trust credential, or change the
parent workspace. Keep its lock file tracked.

Scoped verification:

```
cargo check --manifest-path experimental/ac-av-probe/parallel/825/Cargo.toml
cargo test --manifest-path experimental/ac-av-probe/parallel/825/Cargo.toml av_runtime_825 -- --nocapture
cargo clippy --manifest-path experimental/ac-av-probe/parallel/825/Cargo.toml --all-targets -- -D warnings
```

The tests prove the V-13 module subset: checked-input derivation, O(1) block-PC
lookup, ordered parameter maps, preallocation budget rejection and rollback,
resident code/table identity reuse and old-owner lifetime. Allocator failure is
a controlled failure at the real table reservation boundary, not induced OS
memory exhaustion. The parameter microcase calls the delivered production
`MirFunction::param_slots`; it is not an AV execution benchmark.

The caller supplies remaining memory after existing resident payload charges;
the index slice is additionally capped at the 8 MiB resident partition. Module
accounting covers table capacities, table/container headers and resident Arc
counters. It is not a process peak measurement, nor a second budget for MIR.
The ledger object itself, existing code payload and allocator overhead still
belong in the production consumer's managed/process accounting.

Data and owned profiles are explicitly rejected by this scalar index builder.
Their delivered reference execution remains untouched. Integrating the module
must keep those reference paths, current grants, fuel, cleanup and all three
snapshot execution entry points. No production speed/cache/restore/AutoDown or
V-12 incremental performance criterion is established by these tests.

Integration is blocked on the shared-core handoff after PLAN-822 delivery.
The production adapter must consume this same source, retain the original
FunctionCode identity, reuse the SAME budget domain for retained indices and
combine charges with the existing resident/staging ledger. It must register
this as a private module, never export a portable or executable credential.
